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

use crate::distances::Method;
use crate::maped_distance::MapedDistance;
use controller::{Controller, Entity};
use knn::{Knn, MaxHeapKnn, MinHeapKnn};
use std::marker::PhantomData;

pub struct Engine<'a, C, User, Item>
where
    User: Entity,
    Item: Entity,
    C: Controller<User, Item>,
{
    controller: &'a C,

    user_type: PhantomData<User>,
    item_type: PhantomData<Item>,
}

impl<'a, C, User, Item> Engine<'a, C, User, Item>
where
    User: Entity,
    Item: Entity,
    C: Controller<User, Item>,
{
    pub fn with_controller(controller: &'a C) -> Self {
        Self {
            controller,
            user_type: PhantomData,
            item_type: PhantomData,
        }
    }

    pub fn distance(&self, user_a: &User, user_b: &User, method: distances::Method) -> Option<f64> {
        let rating_a = self.controller.ratings_by(user_a).ok()?;
        let rating_b = self.controller.ratings_by(user_b).ok()?;

        distances::distance(&rating_a, &rating_b, method)
    }

    pub fn knn(
        &self,
        k: usize,
        user: &User,
        method: Method,
        chunk_size: Option<usize>,
    ) -> Option<Vec<(String, f64)>> {
        if k == 0 {
            return None;
        }

        let user_ratings = self.controller.ratings_by(user).ok()?;
        let mut knn: Box<dyn Knn> = if method.is_similarity() {
            Box::new(MinHeapKnn::new(k, method))
        } else {
            Box::new(MaxHeapKnn::new(k, method))
        };

        if let Some(chunk_size) = chunk_size {
            let users_chunks = self.controller.users_by_chunks(chunk_size);
            for users in users_chunks {
                let maped_ratings = self.controller.maped_ratings_by(&users).ok()?;
                knn.update(&user_ratings, maped_ratings);
            }
        } else {
            let maped_ratings = self.controller.maped_ratings_except(user).ok()?;
            knn.update(&user_ratings, maped_ratings);
        }

        Some(
            knn.into_vec()
                .into_iter()
                .map(|MapedDistance(id, dist, _)| (id, dist))
                .collect(),
        )
    }

    pub fn predict(
        &self,
        k: usize,
        user: &User,
        item: &Item,
        method: Method,
        chunk_size: Option<usize>,
    ) -> Option<f64> {
        let item_id = item.get_id();
        let user_ratings = self.controller.ratings_by(user).ok()?;
        let mut knn: Box<dyn Knn> = if method.is_similarity() {
            Box::new(MinHeapKnn::new(k, method))
        } else {
            Box::new(MaxHeapKnn::new(k, method))
        };

        if let Some(chunk_size) = chunk_size {
            let users_chunks = self.controller.users_by_chunks(chunk_size);
            for users in users_chunks {
                let maped_ratings = self
                    .controller
                    .maped_ratings_by(&users)
                    .ok()?
                    .into_iter()
                    .filter(|(_, ratings)| ratings.contains_key(&item_id))
                    .collect();

                knn.update(&user_ratings, maped_ratings);
            }
        } else {
            let maped_ratings = self
                .controller
                .maped_ratings_except(user)
                .ok()?
                .into_iter()
                .filter(|(_id, ratings)| ratings.contains_key(&item_id))
                .collect();

            knn.update(&user_ratings, maped_ratings);
        }

        let pearson_knn: Vec<_> = knn
            .into_vec()
            .into_iter()
            .filter_map(
                |MapedDistance(id, _, ratings)| -> Option<(MapedDistance, f64)> {
                    let nn_ratings = ratings?;

                    if !nn_ratings.contains_key(&item_id) {
                        return None;
                    }

                    let coef = distances::distance(
                        &user_ratings,
                        &nn_ratings,
                        Method::PearsonApproximation,
                    )?;

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

        prediction
    }
}

#[cfg(test)]
mod tests {
    use super::distances::Method;
    use super::*;
    use anyhow::Error;
    use books::BooksController;
    use controller::SearchBy;
    use simple_movie::SimpleMovieController;

    #[test]
    fn euclidean_distance() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(&controller);

        let user_a = &controller.users(&SearchBy::id("52"))?[0];
        let user_b = &controller.users(&SearchBy::id("53"))?[0];

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

        let user_a = &controller.users(&SearchBy::id("52"))?[0];
        let user_b = &controller.users(&SearchBy::id("53"))?[0];

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

        let user_a = &controller.users(&SearchBy::id("52"))?[0];
        let user_b = &controller.users(&SearchBy::id("53"))?[0];

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

        let user = &controller.users(&SearchBy::id("52"))?[0];

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

        let user = &controller.users(&SearchBy::id("52"))?[0];

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

        let user = &controller.users(&SearchBy::id("52"))?[0];

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

        let user = &controller.users(&SearchBy::id("242"))?[0];

        println!(
            "kNN(242, 5, manhattan): {:?}",
            engine.knn(5, user, Method::JaccardDistance, None)
        );

        Ok(())
    }
}
