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

use controller::{Controller, Entity, Id};
use std::{
    cmp::{Ordering, PartialOrd},
    collections::BinaryHeap,
    marker::PhantomData,
};

pub mod distances;

#[derive(Debug, Clone)]
pub struct MapedDistance(Id, f64);

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

pub struct Engine<C, User, Item>
where
    User: Entity,
    Item: Entity,
    C: Controller<User, Item>,
{
    controller: C,

    user_type: PhantomData<User>,
    item_type: PhantomData<Item>,
}

impl<C, User, Item> Engine<C, User, Item>
where
    User: Entity,
    Item: Entity,
    C: Controller<User, Item>,
{
    pub fn with_controller(controller: C) -> Self {
        Self {
            controller,
            user_type: PhantomData,
            item_type: PhantomData,
        }
    }

    pub fn distance(&self, id_a: &Id, id_b: &Id, method: distances::Method) -> Option<f64> {
        let user_a = self.controller.user_by_id(id_a).ok()?;
        let user_b = self.controller.user_by_id(id_b).ok()?;

        let rating_a = self.controller.ratings_by_user(&user_a).ok()?;
        let rating_b = self.controller.ratings_by_user(&user_b).ok()?;

        distances::distance(&rating_a, &rating_b, method)
    }

    pub fn knn(&self, id: &Id, k: usize, method: distances::Method) -> Option<Vec<MapedDistance>> {
        let user = self.controller.user_by_id(id).ok()?;
        let user_rating = self.controller.ratings_by_user(&user).ok()?;
        let maped_ratings = self.controller.ratings_except_for(&user).ok()?;

        let mut max_heap: BinaryHeap<MapedDistance> = BinaryHeap::with_capacity(k);
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
        let engine = Engine::with_controller(controller);

        println!(
            "euclidean(52, 53): {:?}",
            engine.distance(&52.into(), &53.into(), Method::Euclidean)
        );

        Ok(())
    }

    #[test]
    fn manhattan_distance() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(controller);

        println!(
            "manhattan(52, 53): {:?}",
            engine.distance(&52.into(), &53.into(), Method::Manhattan)
        );

        Ok(())
    }

    #[test]
    fn knn_with_manhattan() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(controller);

        println!(
            "kNN(52, manhattan): {:?}",
            engine.knn(&52.into(), 3, Method::Manhattan)
        );

        Ok(())
    }

    #[test]
    fn knn_with_euclidean() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;
        let engine = Engine::with_controller(controller);

        println!(
            "kNN(52, 5, euclidean): {:?}",
            engine.knn(&52.into(), 3, Method::Euclidean)
        );

        Ok(())
    }

    #[test]
    fn knn_in_books() -> Result<(), Error> {
        let controller = BooksController::new()?;
        let engine = Engine::with_controller(controller);

        println!(
            "kNN(242, 5, manhattan): {:?}",
            engine.knn(&242.into(), 5, Method::JaccardDistance)
        );

        Ok(())
    }
}
