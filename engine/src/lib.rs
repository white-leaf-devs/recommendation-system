// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub mod chunked_matrix;
pub mod distances;
pub mod error;
pub mod knn;
pub mod maped_distance;
pub mod utils;

use crate::{
    distances::items::Method as ItemMethod, distances::users::Method as UserMethod,
    maped_distance::MapedDistance,
};
use anyhow::Error;
use config::Config;
use controller::{eid, maped_ratings, Controller, Entity, Ratings};
use distances::items::{denormalize_user_rating, normalize_user_ratings, slope_one, AdjCosine};
use error::ErrorKind;
use knn::{Knn, MaxHeapKnn, MinHeapKnn};
use num_traits::Zero;
use std::{collections::HashSet, fmt::Debug, hash::Hash, marker::PhantomData, time::Instant};

pub struct Engine<'a, C, U, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq,
{
    config: &'a Config,
    controller: &'a C,

    adj_cosine: AdjCosine<eid!(U), f64>,

    user_type: PhantomData<U>,
    item_type: PhantomData<I>,
}

impl<'a, C, U, I> Engine<'a, C, U, I>
where
    C: Controller<User = U, Item = I>,
    U: Entity,
    I: Entity,
    eid!(U): Hash + Eq + Clone + Debug + Default,
    eid!(I): Hash + Eq + Clone + Debug,
{
    pub fn with_controller(controller: &'a C, config: &'a Config) -> Self {
        Self {
            config,
            controller,
            adj_cosine: AdjCosine::new(),
            user_type: PhantomData,
            item_type: PhantomData,
        }
    }

    pub fn user_distance(&self, user_a: U, user_b: U, method: UserMethod) -> Result<f64, Error> {
        let rating_a = self.controller.ratings_by(&user_a)?;
        let rating_b = self.controller.ratings_by(&user_b)?;

        distances::users::distance(&rating_a, &rating_b, method).map_err(Into::into)
    }

    pub fn item_distance(
        &mut self,
        item_a: I,
        item_b: I,
        method: ItemMethod,
    ) -> Result<f64, Error> {
        match method {
            ItemMethod::AdjCosine => {
                let item_a_id = item_a.get_id();
                let item_b_id = item_b.get_id();

                let users_who_rated = self.controller.users_who_rated(&[item_a, item_b])?;

                let all_users_iter = users_who_rated.values();
                let mut all_users = HashSet::new();

                for users in all_users_iter {
                    for user in users.keys() {
                        all_users.insert(user.clone());
                    }
                }

                self.adj_cosine.shrink_means();

                let all_users: Vec<_> = all_users
                    .into_iter()
                    .filter(|uid| !self.adj_cosine.has_mean_for(uid))
                    .collect();

                let all_partial_users = self.controller.create_partial_users(&all_users)?;

                let partial_users_chunk_size = self.config.engine.partial_users_chunk_size;
                for partial_users_chunk in all_partial_users.chunks(partial_users_chunk_size) {
                    let mean_chunk = self.controller.means_for(partial_users_chunk)?;
                    self.adj_cosine.add_new_means(&mean_chunk);
                }

                let sim = self
                    .adj_cosine
                    .calculate(&users_who_rated[&item_a_id], &users_who_rated[&item_b_id])?;

                Ok(sim)
            }

            ItemMethod::SlopeOne => {
                let item_a_id = item_a.get_id();
                let item_b_id = item_b.get_id();
                let users_who_rated = self.controller.users_who_rated(&[item_a, item_b])?;
                let (dev, _) =
                    slope_one(&users_who_rated[&item_a_id], &users_who_rated[&item_b_id])?;

                Ok(dev)
            }
        }
    }

    pub fn user_knn(
        &self,
        k: usize,
        user: U,
        method: UserMethod,
        chunk_size: Option<usize>,
    ) -> Result<Vec<(eid!(U), f64)>, Error> {
        if k == 0 {
            return Err(ErrorKind::EmptyKNearestNeighbors.into());
        }

        let user_ratings = self.controller.ratings_by(&user)?;
        let mut knn: Box<dyn Knn<eid!(U), eid!(I)>> = if method.is_similarity() {
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
            let maped_ratings = self.controller.maped_ratings_except(&user)?;
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

    pub fn user_based_predict(
        &self,
        k: usize,
        user: U,
        item: I,
        method: UserMethod,
        chunk_size: Option<usize>,
    ) -> Result<f64, Error> {
        let item_id = item.get_id();
        let user_ratings = self.controller.ratings_by(&user)?;

        let mut knn: Box<dyn Knn<eid!(U), eid!(I)>> = if method.is_similarity() {
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
                .maped_ratings_except(&user)?
                .into_iter()
                .filter(|(_id, ratings)| ratings.contains_key(&item_id))
                .collect();

            knn.update(&user_ratings, maped_ratings);
        }

        #[allow(clippy::type_complexity)]
        let pearson_knn: Vec<_> = knn
            .into_vec()
            .into_iter()
            .filter_map(
                |MapedDistance(id, _, ratings)| -> Option<(MapedDistance<eid!(U), eid!(I)>, f64)> {
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

    fn adj_cosine_predict(&self, user: U, item: I, chunk_size: usize) -> Result<f64, Error> {
        let user_id = user.get_id();
        let item_id = item.get_id();

        log::info!(
            "Predicting score for user({:?}) for item({:?})",
            user_id,
            item_id
        );

        log::info!("Gathering user({:?}) ratings", user_id);
        let user_ratings = self.controller.ratings_by(&user)?;
        let (min_rating, max_rating) = self.controller.score_range();
        log::info!("Normalizing user({:?}) ratings", user_id);
        let normalized_ratings = normalize_user_ratings(&user_ratings, min_rating, max_rating)?;

        log::info!("Gathering users who rated for target item");
        let target_items_users = self.controller.users_who_rated(&[item])?;
        log::info!(
            "Gathered {} scores for this item",
            target_items_users[&item_id].len()
        );

        let mut num = 0.0;
        let mut dem = 0.0;

        let mut adj_cosine = AdjCosine::new();

        let mut means_time = 0.0;
        let mut iters_time = 0.0;
        let mut uwrs_time = 0.0;

        log::info!("Iterating items by chunks of size {}", chunk_size);
        let items_chunks = self.controller.items_by_chunks(chunk_size);
        for item_chunk_base in items_chunks {
            log::info!("Initial chunk size: {}", item_chunk_base.len());
            let now = Instant::now();
            let item_chunk: Vec<_> = item_chunk_base
                .into_iter()
                .filter(|other_item| user_ratings.contains_key(&other_item.get_id()))
                .collect();
            log::info!("Chunk size after filter: {}", item_chunk.len());
            log::info!(
                "Filtering chunk took {} seconds",
                now.elapsed().as_secs_f64()
            );

            if item_chunk.is_empty() {
                continue;
            }

            log::info!("Gathering users who rated for {} items", item_chunk.len());
            let now = Instant::now();
            let mut users_who_rated: maped_ratings!(I => U) = self
                .controller
                .users_who_rated(&item_chunk)?
                .into_iter()
                .filter(|(_, ratings)| ratings.contains_key(&user_id))
                .collect();

            let uwr_time = now.elapsed().as_secs_f64();
            uwrs_time += uwr_time;
            log::info!(
                "Gathered a total of {} inverted maped ratings",
                users_who_rated.len()
            );
            log::info!("Gathering users who rated took {} seconds", uwr_time);

            users_who_rated.insert(item_id.clone(), target_items_users[&item_id].clone());

            let all_users_iter = users_who_rated.values();
            let mut all_users = HashSet::new();

            log::info!("Collecting all unique users ids");
            for users in all_users_iter {
                for user_id in users.keys() {
                    all_users.insert(user_id.clone());
                }
            }

            // Shrink some means by their usage frequency
            log::info!("Shrinking means based on their usage");
            adj_cosine.shrink_means();

            // Collect all the users that doesn't have a calculated mean
            log::info!("Filtering users that have a cached mean");
            let all_users: Vec<_> = all_users
                .into_iter()
                .filter(|user_id| !adj_cosine.has_mean_for(user_id))
                .collect();
            let all_partial_users = self.controller.create_partial_users(&all_users)?;

            log::info!("Gathering means for {} users", all_partial_users.len());
            let now = Instant::now();
            let partial_users_chunk_size = self.config.engine.partial_users_chunk_size;
            for partial_users_chunk in all_partial_users.chunks(partial_users_chunk_size) {
                let mean_chunk = self.controller.means_for(partial_users_chunk)?;
                adj_cosine.add_new_means(&mean_chunk);
            }
            let mean_time = now.elapsed().as_secs_f64();
            log::info!("Obtaining took {} seconds", mean_time);
            means_time += mean_time;

            log::info!("Iterating over all the items of this chunk");
            let now = Instant::now();
            for other_item in &item_chunk {
                let other_item_id = other_item.get_id();

                if !normalized_ratings.contains_key(&other_item_id) || item_id == other_item_id {
                    continue;
                }

                if let Ok(similarity) = adj_cosine
                    .calculate(&users_who_rated[&item_id], &users_who_rated[&other_item_id])
                {
                    num += similarity * normalized_ratings[&other_item_id];
                    dem += similarity.abs();
                }
            }

            let iter_time = now.elapsed().as_secs_f64();
            log::info!("Iterating over the items took {} seconds", iter_time);
            iters_time += iter_time;
        }

        log::info!("Gathering means took in total {} seconds", means_time);
        log::info!(
            "Gathering users who rated took in total {} seconds",
            uwrs_time
        );
        log::info!("Computing distances took in total {} seconds", iters_time);
        if dem.is_zero() {
            return Err(ErrorKind::DivisionByZero.into());
        }

        log::info!("Denormalizing the final score");
        Ok(denormalize_user_rating(num / dem, min_rating, max_rating)?)
    }

    pub fn slope_one_predict(&self, user: U, item: I, chunk_size: usize) -> Result<f64, Error> {
        let target_item_id = item.get_id();
        let target_item_ratings = &self.controller.users_who_rated(&[item])?[&target_item_id];

        let user_ratings: Ratings<_, _> = self
            .controller
            .ratings_by(&user)?
            .into_iter()
            .filter(|(id, _)| id != &target_item_id)
            .collect();

        let items_ids: Vec<_> = user_ratings.iter().map(|(id, _)| id.to_owned()).collect();
        let all_partial_items = self.controller.create_partial_items(&items_ids)?;

        let mut num = 0.0;
        let mut den = 0.0;

        for partial_items_chunk in all_partial_items.chunks(chunk_size) {
            let users_who_rated = self.controller.users_who_rated(partial_items_chunk)?;
            for (item_id, ratings) in users_who_rated {
                if let Ok((dev, card)) = slope_one(target_item_ratings, &ratings) {
                    num += (dev + user_ratings[&item_id]) * card as f64;
                    den += card as f64;
                }
            }
        }

        if den.is_zero() {
            Err(ErrorKind::DivisionByZero.into())
        } else {
            Ok(num / den)
        }
    }

    pub fn item_based_predict(
        &self,
        user: U,
        item: I,
        method: ItemMethod,
        chunk_size: usize,
    ) -> Result<f64, Error> {
        match method {
            ItemMethod::AdjCosine => self.adj_cosine_predict(user, item, chunk_size),
            ItemMethod::SlopeOne => self.slope_one_predict(user, item, chunk_size),
        }
    }
}

#[cfg(feature = "test-engine")]
#[cfg(test)]
mod tests {
    use super::distances::users::Method;
    use super::*;
    use anyhow::Error;
    use books::BooksController;
    use config::Config;
    use controller::SearchBy;
    use simple_movie::SimpleMovieController;

    #[test]
    fn euclidean_distance() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user_a = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        let user_b = controller
            .users_by(&SearchBy::id("53"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "euclidean(52, 53): {:?}",
            engine.user_distance(user_a, user_b, Method::Euclidean)
        );

        Ok(())
    }

    #[test]
    fn manhattan_distance() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user_a = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        let user_b = controller
            .users_by(&SearchBy::id("53"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "manhattan(52, 53): {:?}",
            engine.user_distance(user_a, user_b, Method::Manhattan)
        );

        Ok(())
    }

    #[test]
    fn cosine_similarity_distance() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user_a = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        let user_b = controller
            .users_by(&SearchBy::id("53"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "cosine(52, 53): {:?}",
            engine.user_distance(user_a, user_b, Method::CosineSimilarity)
        );

        Ok(())
    }

    #[test]
    fn knn_with_manhattan() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "kNN(52, manhattan): {:?}",
            engine.user_knn(4, user, Method::Manhattan, None)
        );

        Ok(())
    }

    #[test]
    fn knn_with_euclidean() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "kNN(52, 3, euclidean): {:?}",
            engine.user_knn(3, user, Method::Euclidean, None)
        );

        Ok(())
    }

    #[test]
    fn knn_with_cosine() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("52"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "kNN(52, 3, cosine): {:?}",
            engine.user_knn(3, user, Method::CosineSimilarity, None)
        );

        Ok(())
    }

    #[test]
    fn knn_in_books() -> Result<(), Error> {
        let config = Config::default();
        let psql_url = &config.databases["books"].psql_url;
        let mongo_url = &config.databases["books"].mongo_url;
        let mongo_db = &config.databases["books"].mongo_db;

        let controller = BooksController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("242"))?
            .drain(..1)
            .next()
            .unwrap();

        println!(
            "kNN(242, 5, manhattan): {:?}",
            engine.user_knn(5, user, Method::JaccardDistance, None)
        );

        Ok(())
    }

    #[test]
    fn similarity_matrix() -> Result<(), Error> {
        use super::chunked_matrix::{ChunkedMatrix, SimilarityMatrix};
        use movie_lens_small::MovieLensSmallController;
        use std::time::Instant;

        let config = Config::default();
        let psql_url = &config.databases["movie-lens-small"].psql_url;
        let mongo_url = &config.databases["movie-lens-small"].mongo_url;
        let mongo_db = &config.databases["movie-lens-small"].mongo_db;

        let controller = MovieLensSmallController::with_url(psql_url, mongo_url, mongo_db)?;
        let mut sim_matrix = SimilarityMatrix::new(&controller, &config, 10000, 10000);

        let now = Instant::now();
        sim_matrix.calculate_chunk(0, 0)?;
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        Ok(())
    }

    #[test]
    fn item_based_pred() -> Result<(), Error> {
        use books::BooksController;
        use std::time::Instant;

        let config = Config::default();
        let psql_url = &config.databases["books"].psql_url;
        let mongo_url = &config.databases["books"].mongo_url;
        let mongo_db = &config.databases["books"].mongo_db;

        let controller = BooksController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("243"))?
            .drain(..1)
            .next()
            .unwrap();

        let item = controller
            .items_by(&SearchBy::name("Flesh Tones: A Novel"))?
            .drain(..1)
            .next()
            .unwrap();

        let now = Instant::now();
        println!(
            "Item based prediction Books: {:?}",
            engine.item_based_predict(user, item, ItemMethod::SlopeOne, 2500)?
        );
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        let psql_url = &config.databases["simple-movie"].psql_url;
        let mongo_url = &config.databases["simple-movie"].mongo_url;
        let mongo_db = &config.databases["simple-movie"].mongo_db;

        let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::name("Josh"))?
            .drain(..1)
            .next()
            .unwrap();

        let item = controller
            .items_by(&SearchBy::name("Blade Runner"))?
            .drain(..1)
            .next()
            .unwrap();

        let now = Instant::now();
        println!(
            "\nItem based prediction SimpleMovie: {:?}",
            engine.item_based_predict(user, item, ItemMethod::SlopeOne, 2500)?
        );
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        use movie_lens_small::MovieLensSmallController;

        let psql_url = &config.databases["movie-lens-small"].psql_url;
        let mongo_url = &config.databases["movie-lens-small"].mongo_url;
        let mongo_db = &config.databases["movie-lens-small"].mongo_db;

        let controller = MovieLensSmallController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("2"))?
            .drain(..1)
            .next()
            .unwrap();

        let item = controller
            .items_by(&SearchBy::name("Suture (1993)"))?
            .drain(..1)
            .next()
            .unwrap();

        let now = Instant::now();
        println!(
            "\nItem based prediction MovieLensSmall: {:?}",
            engine.item_based_predict(user, item, ItemMethod::SlopeOne, 2500)?
        );
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        use movie_lens::MovieLensController;

        let psql_url = &config.databases["movie-lens"].psql_url;
        let mongo_url = &config.databases["movie-lens"].mongo_url;
        let mongo_db = &config.databases["movie-lens"].mongo_db;

        let controller = MovieLensController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("35826"))?
            .drain(..1)
            .next()
            .unwrap();

        let item = controller
            .items_by(&SearchBy::id("307"))?
            .drain(..1)
            .next()
            .unwrap();

        let now = Instant::now();
        println!(
            "\nItem based prediction MovieLens: {:?}",
            engine.item_based_predict(user, item, ItemMethod::SlopeOne, 2500)?
        );
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        Ok(())
    }

    #[test]
    #[ignore]
    fn shelves_item_based_pred() -> Result<(), Error> {
        use shelves::ShelvesController;
        use std::time::Instant;

        let config = Config::default();
        let psql_url = &config.databases["shelves"].psql_url;
        let mongo_url = &config.databases["shelves"].mongo_url;
        let mongo_db = &config.databases["shelves"].mongo_db;

        let controller = ShelvesController::with_url(psql_url, mongo_url, mongo_db)?;
        let engine = Engine::with_controller(&controller, &config);

        let user = controller
            .users_by(&SearchBy::id("0"))?
            .drain(..1)
            .next()
            .unwrap();

        let item = controller
            .items_by(&SearchBy::id("1000"))?
            .drain(..1)
            .next()
            .unwrap();

        let now = Instant::now();
        println!(
            "Item based prediction (UserId 0, ItemId 1000, 1): {:?}",
            engine.item_based_predict(user, item, ItemMethod::AdjCosine, 1)?
        );
        println!("Elapsed: {}", now.elapsed().as_secs_f64());

        Ok(())
    }
}
