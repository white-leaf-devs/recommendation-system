use crate::{
    distances::{self, users::Method},
    maped_distance::MapedDistance,
};
use controller::{MapedRatings, Ratings};
use std::{cmp::Reverse, collections::BinaryHeap, hash::Hash};

type MaxHeap<T> = BinaryHeap<T>;
type MinHeap<T> = BinaryHeap<Reverse<T>>;

pub trait Knn<UserId, ItemId> {
    fn update(&mut self, user_ratings: &Ratings<ItemId>, maped_ratings: MapedRatings<UserId, ItemId>);
    fn into_vec(self: Box<Self>) -> Vec<MapedDistance<UserId, ItemId>>;
}

pub struct MaxHeapKnn<UserId, ItemId> {
    k: usize,
    method: Method,
    max_heap: MaxHeap<MapedDistance<UserId, ItemId>>,
}

impl<UserId, ItemId> MaxHeapKnn<UserId, ItemId> {
    pub fn new(k: usize, method: Method) -> Self {
        Self {
            k,
            method,
            max_heap: Default::default(),
        }
    }
}

impl<UserId, ItemId> Knn<UserId, ItemId> for MaxHeapKnn<UserId, ItemId> 
where
    UserId: Hash + Eq,
    ItemId: Hash + Eq
{
    fn update(&mut self, user_ratings: &Ratings<ItemId>, maped_ratings: MapedRatings<UserId, ItemId>) {
        for (user_id, ratings) in maped_ratings {
            let distance = distances::users::distance(user_ratings, &ratings, self.method);

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

    fn into_vec(self: Box<Self>) -> Vec<MapedDistance<UserId, ItemId>> {
        self.max_heap.into_sorted_vec()
    }
}

pub struct MinHeapKnn<UserId, ItemId> {
    k: usize,
    method: Method,
    min_heap: MinHeap<MapedDistance<UserId, ItemId>>,
}

impl<UserId, ItemId> MinHeapKnn<UserId, ItemId> {
    pub fn new(k: usize, method: Method) -> Self {
        Self {
            k,
            method,
            min_heap: Default::default(),
        }
    }
}

impl<UserId, ItemId> Knn<UserId, ItemId> for MinHeapKnn<UserId, ItemId> 
where
    UserId: Hash + Eq,
    ItemId: Hash + Eq
{
    fn update(&mut self, user_ratings: &Ratings<ItemId>, maped_ratings: MapedRatings<UserId, ItemId>) {
        for (user_id, ratings) in maped_ratings {
            let distance = distances::users::distance(user_ratings, &ratings, self.method);

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

    fn into_vec(self: Box<Self>) -> Vec<MapedDistance<UserId, ItemId>> {
        self.min_heap
            .into_sorted_vec()
            .into_iter()
            .map(|r| r.0)
            .collect()
    }
}
