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

use self::distances::Method;
use controller::{Controller, Entity, SearchBy};
use std::{
    cmp::{Ordering, PartialOrd, Reverse},
    collections::BinaryHeap,
    marker::PhantomData,
};

#[derive(Debug, Clone)]
pub struct MapedDistance(pub String, pub f64);

impl PartialEq for MapedDistance {
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl Eq for MapedDistance {}

impl PartialOrd for MapedDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for MapedDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.partial_cmp(&other.1).unwrap()
    }
}

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

    fn max_heap_knn(&self, k: usize, user: &User, method: Method) -> Option<Vec<MapedDistance>> {
        let user_rating = self.controller.ratings_by(&user).ok()?;
        let maped_ratings = self.controller.ratings_except(&user).ok()?;

        let mut max_heap = BinaryHeap::with_capacity(k);
        for (user_id, rating) in maped_ratings {
            let distance = distances::distance(&user_rating, &rating, method);

            if let Some(distance) = distance {
                if max_heap.len() < k {
                    max_heap.push(MapedDistance(user_id, distance));
                } else {
                    let maximum = max_heap.peek()?;
                    if distance < maximum.1 {
                        max_heap.pop();
                        max_heap.push(MapedDistance(user_id, distance));
                    }
                }
            }
        }

        Some(max_heap.into_sorted_vec())
    }

    fn min_heap_knn(&self, k: usize, user: &User, method: Method) -> Option<Vec<MapedDistance>> {
        let user_rating = self.controller.ratings_by(&user).ok()?;
        let maped_ratings = self.controller.ratings_except(&user).ok()?;

        let mut min_heap = BinaryHeap::with_capacity(k);
        for (user_id, rating) in maped_ratings {
            let distance = distances::distance(&user_rating, &rating, method);

            if let Some(distance) = distance {
                if min_heap.len() < k {
                    min_heap.push(Reverse(MapedDistance(user_id, distance)));
                } else {
                    let minimum = min_heap.peek()?;
                    if distance > (minimum.0).1 {
                        min_heap.pop();
                        min_heap.push(Reverse(MapedDistance(user_id, distance)));
                    }
                }
            }
        }

        Some(
            min_heap
                .into_sorted_vec()
                .into_iter()
                .map(|r| r.0)
                .collect(),
        )
    }

    pub fn knn(&self, k: usize, user: &User, method: Method) -> Option<Vec<MapedDistance>> {
        match method {
            Method::Manhattan
            | Method::Euclidean
            | Method::Minkowski(_)
            | Method::JaccardDistance => self.max_heap_knn(k, user, method),

            Method::CosineSimilarity
            | Method::PearsonCorrelation
            | Method::JaccardIndex
            | Method::PearsonApproximation => self.min_heap_knn(k, user, method),
        }
    }

    pub fn predict(&self, k: usize, user: &User, item: &Item, method: Method) -> Option<f64> {
        let relevant_knn = self.knn(k, user, method)?;
        let item_id = item.get_id();

        let pearson_knn: Vec<_> = relevant_knn
            .into_iter()
            .filter_map(|MapedDistance(nn_id, _)| -> Option<MapedDistance> {
                let nn_user = &self.controller.users(&SearchBy::id(&nn_id)).ok()?[0];

                // Check if the nn contains an score for the given item
                let nn_ratings = self.controller.ratings_by(nn_user).ok()?;
                // If not, return early since we don't need this nn
                if !nn_ratings.contains_key(&item_id) {
                    return None;
                }

                let distance = self.distance(user, &nn_user, Method::PearsonApproximation)?;

                if distance > 0.0 {
                    Some(MapedDistance(nn_id, distance))
                } else {
                    None
                }
            })
            .collect();

        let total = pearson_knn
            .iter()
            .fold(0.0, |acc, MapedDistance(_, dist)| acc + dist);

        let mut prediction = None;
        for MapedDistance(nn_id, distance) in pearson_knn {
            let nn_user = &self.controller.users(&SearchBy::id(&nn_id)).ok()?[0];

            let nn_ratings = self.controller.ratings_by(nn_user).ok()?;
            let nn_item_rating = nn_ratings.get(&item_id)?;

            *prediction.get_or_insert(0.0) += nn_item_rating * (distance / total);
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
            engine.knn(4, user, Method::Manhattan)
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
            engine.knn(3, user, Method::Euclidean)
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
            engine.knn(3, user, Method::CosineSimilarity)
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
            engine.knn(5, user, Method::JaccardDistance)
        );

        Ok(())
    }
}
