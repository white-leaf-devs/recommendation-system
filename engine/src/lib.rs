// Copyright (C) 2020 Kevin Del Castillo Ramírez
//
// This file is part of recommend.
//
// recommend is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// recommend is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with recommend.  If not, see <http://www.gnu.org/licenses/>.

pub mod distances;
pub mod knn;
pub mod maped_distance;
pub mod similarity_matrix;
pub mod utils;

use crate::{distances::users::Method as UserMethod, maped_distance::MapedDistance};
use anyhow::Error;
use controller::{Controller, Entity, ItemsUsers};
use distances::{
    error::ErrorKind,
    items::{
        adjusted_cosine_means, denormalize_user_rating, fast_adjusted_cosine,
        normalize_user_ratings,
    },
};
use knn::{Knn, MaxHeapKnn, MinHeapKnn};
use std::{collections::HashSet, hash::Hash, marker::PhantomData};

pub struct Engine<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
    C: Controller<User, UserId, Item, ItemId>,
{
    controller: &'a C,

    user_type: PhantomData<User>,
    item_type: PhantomData<Item>,
}

impl<'a, C, User, UserId, Item, ItemId> Engine<'a, C, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId> + Clone,
    UserId: Hash + Eq + Clone,
    ItemId: Hash + Eq + Clone,
    C: Controller<User, UserId, Item, ItemId>,
{
    pub fn with_controller(controller: &'a C) -> Self {
        Self {
            controller,
            user_type: PhantomData,
            item_type: PhantomData,
        }
    }

    pub fn user_distance(
        &self,
        user_a: &User,
        user_b: &User,
        method: UserMethod,
    ) -> Result<f64, Error> {
        let rating_a = self.controller.ratings_by(user_a)?;
        let rating_b = self.controller.ratings_by(user_b)?;

        distances::users::distance(&rating_a, &rating_b, method).map_err(Into::into)
    }

    pub fn user_knn(
        &self,
        k: usize,
        user: &User,
        method: UserMethod,
        chunk_size: Option<usize>,
    ) -> Result<Vec<(UserId, f64)>, Error> {
        if k == 0 {
            return Err(ErrorKind::EmptyKNearestNeighbors.into());
        }

        let user_ratings = self.controller.ratings_by(user)?;
        let mut knn: Box<dyn Knn<UserId, ItemId>> = if method.is_similarity() {
            Box::new(MinHeapKnn::new(k, method))
        } else {
            Box::new(MaxHeapKnn::new(k, method))
        };

        if let Some(chunk_size) = chunk_size {
            let users_chunks = self.controller.users_by_chunks(chunk_size);
            for users in users_chunks {
                let maped_ratings = self.controller.maped_ratings_by(&users)?;
                knn.update(&user_ratings, maped_ratings);
            }
        } else {
            let maped_ratings = self.controller.maped_ratings_except(user)?;
            knn.update(&user_ratings, maped_ratings);
        }

        let knn: Vec<_> = knn
            .into_vec()
            .into_iter()
            .map(|MapedDistance(id, dist, _)| (id, dist))
            .collect();

        if knn.is_empty() {
            Err(ErrorKind::EmptyKNearestNeighbors.into())
        } else {
            Ok(knn)
        }
    }

    pub fn user_predict(
        &self,
        k: usize,
        user: &User,
        item: &Item,
        method: UserMethod,
        chunk_size: Option<usize>,
    ) -> Result<f64, Error> {
        let item_id = item.get_id();
        let user_ratings = self.controller.ratings_by(user)?;
        let mut knn: Box<dyn Knn<UserId, ItemId>> = if method.is_similarity() {
            Box::new(MinHeapKnn::new(k, method))
        } else {
            Box::new(MaxHeapKnn::new(k, method))
        };

        if let Some(chunk_size) = chunk_size {
            let users_chunks = self.controller.users_by_chunks(chunk_size);
            for users in users_chunks {
                let maped_ratings = self
                    .controller
                    .maped_ratings_by(&users)?
                    .into_iter()
                    .filter(|(_, ratings)| ratings.contains_key(&item_id))
                    .collect();

                knn.update(&user_ratings, maped_ratings);
            }
        } else {
            let maped_ratings = self
                .controller
                .maped_ratings_except(user)?
                .into_iter()
                .filter(|(_id, ratings)| ratings.contains_key(&item_id))
                .collect();

            knn.update(&user_ratings, maped_ratings);
        }

        let pearson_knn: Vec<_> = knn
            .into_vec()
            .into_iter()
            .filter_map(
                |MapedDistance(id, _, ratings)| -> Option<(MapedDistance<UserId, ItemId>, f64)> {
                    let nn_ratings = ratings?;

                    if !nn_ratings.contains_key(&item_id) {
                        return None;
                    }

                    let coef = distances::users::distance(
                        &user_ratings,
                        &nn_ratings,
                        UserMethod::PearsonApproximation,
                    )
                    .ok()?;

                    Some((MapedDistance(id, coef, None), *nn_ratings.get(&item_id)?))
                },
            )
            .collect();

        let total = pearson_knn
            .iter()
            .fold(0.0, |acc, (maped_distance, _)| acc + maped_distance.dist());

        let mut prediction = None;
        for (maped_distance, nn_rating) in pearson_knn {
            *prediction.get_or_insert(0.0) += nn_rating * (maped_distance.dist() / total);
        }

        prediction.ok_or_else(|| ErrorKind::EmptyKNearestNeighbors.into())
    }

    pub fn item_based_prediction(
        &self,
        user: &User,
        item: &Item,
        chunk_size: usize,
    ) -> Result<f64, Error> {
        let item_id = item.get_id();

        let all_items_it = self.controller.items_by_chunks(chunk_size);

        let user_ratings = self.controller.ratings_by(user)?;
        let normalized_ratings = normalize_user_ratings(&user_ratings, 1.0, 5.0)?;

        //let target_item_users = &self.controller.users_who_rated(&[item.clone()])?[&item_id];

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for mut item_chunk in all_items_it {
            item_chunk.push(item.clone());
            let users_who_rated: ItemsUsers<ItemId, UserId> = self
                .controller
                .users_who_rated(&item_chunk)?
                .into_iter()
                .filter(|(other_item_id, set)| {
                    set.contains(&user.get_id()) || item_id == *other_item_id
                })
                .collect();

            let all_users: Vec<UserId> = users_who_rated
                .values()
                .into_iter()
                .fold(HashSet::new(), |acc, other_set| {
                    acc.union(other_set).cloned().collect()
                })
                .into_iter()
                .collect();

            let all_partial_users = self.controller.create_partial_users(&all_users)?;

            let maped_ratings = self.controller.maped_ratings_by(&all_partial_users)?;
            let means = adjusted_cosine_means(&maped_ratings);

            for other_item in &item_chunk {
                let other_item_id = other_item.get_id();
                if !normalized_ratings.contains_key(&other_item_id) || item_id == other_item_id {
                    continue;
                }

                if let Some(similarity) = fast_adjusted_cosine(
                    &means,
                    &maped_ratings,
                    &users_who_rated[&item_id],
                    &users_who_rated[&other_item_id],
                    &item_id,
                    &other_item_id,
                ) {
                    numerator += similarity * normalized_ratings[&other_item_id];
                    denominator += similarity.abs();
                }
            }
        }

        if denominator == 0.0 {
            return Err(ErrorKind::DivisionByZero.into());
        }
        Ok(denormalize_user_rating(numerator / denominator, 1.0, 5.0))
    }
}

#[cfg(feature = "test-engine")]
#[cfg(test)]
mod tests {
    use super::distances::users::Method;
    use super::*;
    use anyhow::Error;
    use books::BooksController;
    use controller::SearchBy;
    use simple_movie::SimpleMovieController;
    /*
    #[test]
    fn euclidean_distance() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user_a = &controller.users_by(&SearchBy::id("52"))?[0];
        let user_b = &controller.users_by(&SearchBy::id("53"))?[0];

        println!(
            "euclidean(52, 53): {:?}",
            engine.distance(user_a, user_b, Method::Euclidean)
        );

        Ok(())
    }

    #[test]
    fn manhattan_distance() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user_a = &controller.users_by(&SearchBy::id("52"))?[0];
        let user_b = &controller.users_by(&SearchBy::id("53"))?[0];

        println!(
            "manhattan(52, 53): {:?}",
            engine.distance(user_a, user_b, Method::Manhattan)
        );

        Ok(())
    }

    #[test]
    fn cosine_similarity_distance() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user_a = &controller.users_by(&SearchBy::id("52"))?[0];
        let user_b = &controller.users_by(&SearchBy::id("53"))?[0];

        println!(
            "cosine(52, 53): {:?}",
            engine.distance(user_a, user_b, Method::CosineSimilarity)
        );

        Ok(())
    }

    #[test]
    fn knn_with_manhattan() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user = &controller.users_by(&SearchBy::id("52"))?[0];

        println!(
            "kNN(52, manhattan): {:?}",
            engine.knn(4, user, Method::Manhattan, None)
        );

        Ok(())
    }

    #[test]
    fn knn_with_euclidean() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user = &controller.users_by(&SearchBy::id("52"))?[0];

        println!(
            "kNN(52, 3, euclidean): {:?}",
            engine.knn(3, user, Method::Euclidean, None)
        );

        Ok(())
    }

    #[test]
    fn knn_with_cosine() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user = &controller.users_by(&SearchBy::id("52"))?[0];

        println!(
            "kNN(52, 3, cosine): {:?}",
            engine.knn(3, user, Method::CosineSimilarity, None)
        );

        Ok(())
    }

    #[test]
    fn knn_in_books() -> Result<(), Error> {
        let controller = BooksController::new()?;
        let engine = Engine::with_controller(&controller);

        let user = &controller.users_by(&SearchBy::id("242"))?[0];

        println!(
            "kNN(242, 5, manhattan): {:?}",
            engine.knn(5, user, Method::JaccardDistance, None)
        );

        Ok(())
    }

    #[test]
    fn similarity_matrix() -> Result<(), Error> {
        use movie_lens_small::MovieLensSmallController;
        use std::time::Instant;

        let controller = MovieLensSmallController::new()?;
        let mut sim_matrix = SimilarityMatrix::new(&controller, 10000, 10000, 100);

        let now = Instant::now();
        let _matrix = sim_matrix.get_chunk(0, 0);
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        Ok(())
    }*/

    #[test]
    fn item_based_pred() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user = &controller.users_by(&SearchBy::name("Patrick C"))?[0];
        let item = &controller.items_by(&SearchBy::name("Alien"))?[0];

        println!(
            "Item based prediction (Patrick C, Alien, 100): {:?}",
            engine.item_based_prediction(&user, &item, 100)?
        );

        Ok(())
    }
}
