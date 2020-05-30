use crate::{
    distances::{self, Method},
    maped_distance::MapedDistance,
};
use controller::{MapedRatings, Ratings};
use std::{cmp::Reverse, collections::BinaryHeap};

type MaxHeap<T> = BinaryHeap<T>;
type MinHeap<T> = BinaryHeap<Reverse<T>>;

pub trait Knn {
    fn update(&mut self, user_ratings: &Ratings, maped_ratings: MapedRatings);
    fn into_vec(&self) -> Vec<MapedDistance>;
}

pub struct MaxHeapKnn {
    k: usize,
    method: Method,
    max_heap: MaxHeap<MapedDistance>,
}

impl MaxHeapKnn {
    pub fn new(k: usize, method: Method) -> Self {
        Self {
            k,
            method,
            max_heap: Default::default(),
        }
    }
}

impl Knn for MaxHeapKnn {
    fn update(&mut self, user_ratings: &Ratings, maped_ratings: MapedRatings) {
        for (user_id, ratings) in maped_ratings {
            let distance = distances::distance(user_ratings, &ratings, self.method);

            if let Some(distance) = distance {
                if self.max_heap.len() < self.k {
                    let maped_distance = MapedDistance(user_id, distance, Some(ratings));
                    self.max_heap.push(maped_distance);
                } else {
                    let maximum = self.max_heap.peek().unwrap();
                    if distance < maximum.dist() {
                        let maped_distance = MapedDistance(user_id, distance, Some(ratings));

                        self.max_heap.pop();
                        self.max_heap.push(maped_distance);
                    }
                }
            }
        }
    }

    fn into_vec(&self) -> Vec<MapedDistance> {
        self.max_heap.clone().into_sorted_vec()
    }
}

pub struct MinHeapKnn {
    k: usize,
    method: Method,
    min_heap: MinHeap<MapedDistance>,
}

impl MinHeapKnn {
    pub fn new(k: usize, method: Method) -> Self {
        Self {
            k,
            method,
            min_heap: Default::default(),
        }
    }
}

impl Knn for MinHeapKnn {
    fn update(&mut self, user_ratings: &Ratings, maped_ratings: MapedRatings) {
        for (user_id, ratings) in maped_ratings {
            let distance = distances::distance(user_ratings, &ratings, self.method);

            if let Some(distance) = distance {
                if self.min_heap.len() < self.k {
                    let maped_distance = MapedDistance(user_id, distance, Some(ratings));
                    self.min_heap.push(Reverse(maped_distance));
                } else {
                    let minimum = self.min_heap.peek().unwrap();
                    if distance > (minimum.0).dist() {
                        let maped_distance = MapedDistance(user_id, distance, Some(ratings));

                        self.min_heap.pop();
                        self.min_heap.push(Reverse(maped_distance));
                    }
                }
            }
        }
    }

    fn into_vec(&self) -> Vec<MapedDistance> {
        self.min_heap
            .clone()
            .into_sorted_vec()
            .into_iter()
            .map(|r| r.0)
            .collect()
    }
}
