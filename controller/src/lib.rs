// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub mod entity;
pub mod error;
pub mod lazy;
pub mod searchby;
pub mod values;

#[macro_export]
macro_rules! eid {
    ($e:ty) => {
        <$e as $crate::entity::Entity>::Id
    };
}

#[macro_export]
macro_rules! maped_ratings {
    ($u:ty => $v:ty) => {
        $crate::MapedRatings<$crate::eid!($u), $crate::eid!($v)>
    };
}

#[macro_export]
macro_rules! ratings {
    ($e:ty) => {
        $crate::Ratings<$crate::eid!($e)>
    }
}

#[macro_export]
macro_rules! means {
    ($e:ty) => {
        $crate::Means<$crate::eid!($e)>
    }
}

use anyhow::Error;
use std::collections::HashMap;

pub use entity::{Entity, ToTable};
pub use lazy::{LazyItemChunks, LazyUserChunks};
pub use searchby::SearchBy;
pub use values::{Field, Type, Value};

pub type Result<T> = std::result::Result<T, Error>;
pub type Means<K, Value = f64> = HashMap<K, Value>;
pub type Ratings<I, Value = f64> = HashMap<I, Value>;
pub type MapedRatings<K, I, Value = f64> = HashMap<K, Ratings<I, Value>>;

pub trait Controller {
    type User: Entity;
    type Item: Entity;
    type Rating: Entity;

    /// Get all users
    fn users(&self) -> Result<Vec<Self::User>>;

    /// Get users that matched the search criteria by id, name or custom (if implemented)
    fn users_by(&self, by: &SearchBy) -> Result<Vec<Self::User>>;

    /// Get a chunk of users specified by certain offset and limit
    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::User>>;

    /// Build an iterator that returns all users by chunks
    fn users_by_chunks(&self, chunk_size: usize) -> LazyUserChunks<Self, Self::User>
    where
        Self: Sized,
    {
        LazyUserChunks {
            curr_offset: 0,
            chunk_size,
            controller: self,
        }
    }

    /// Get all items
    fn items(&self) -> Result<Vec<Self::Item>>;

    /// Get items that matched the search criteria by id, name or custom (if implemented)
    fn items_by(&self, by: &SearchBy) -> Result<Vec<Self::Item>>;

    /// Get a chunk of items specified by certain offset and limit
    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::Item>>;

    /// Build an iterator that returns all items by chunks
    fn items_by_chunks(&self, chunk_size: usize) -> LazyItemChunks<Self, Self::Item>
    where
        Self: Sized,
    {
        LazyItemChunks {
            curr_offset: 0,
            chunk_size,
            controller: self,
        }
    }

    /// Build skeleton/partial users, useful to use in other queries
    fn create_partial_users(&self, user_ids: &[eid!(Self::User)]) -> Result<Vec<Self::User>>;

    /// Build skeleton/partial items, useful to use in other queries
    fn create_partial_items(&self, item_ids: &[eid!(Self::Item)]) -> Result<Vec<Self::Item>>;

    /// Get an "inverted" MapedRatings, i.e. maps Item::Id => User::Id
    #[allow(clippy::type_complexity)]
    fn users_who_rated(
        &self,
        items: &[Self::Item],
    ) -> Result<maped_ratings!(Self::Item => Self::User)>;

    /// Get the ratings for the specified user
    fn user_ratings(&self, user: &Self::User) -> Result<ratings!(Self::Item)>;

    /// Get all normal MapedRatings, i.e. maps User::Id => Item::Id
    #[allow(clippy::type_complexity)]
    fn all_users_ratings(&self) -> Result<maped_ratings!(Self::User => Self::Item)>;

    /// Get some normal MapedRatings for the specified users, i.e. maps User::Id => Item::Id
    #[allow(clippy::type_complexity)]
    fn users_ratings(
        &self,
        users: &[Self::User],
    ) -> Result<maped_ratings!(Self::User => Self::Item)>;

    /// Get all normal MapedRatings except for the specified user, i.e. maps User::Id => Item::Id
    #[allow(clippy::type_complexity)]
    fn users_ratings_except(
        &self,
        user: &Self::User,
    ) -> Result<maped_ratings!(Self::User => Self::Item)>;

    /// Get means for the specified users, returns a map of User::Id => f64
    fn users_means(&self, users: &[Self::User]) -> Result<means!(Self::User)>;

    /// The controller score range, ex. (0.0, 5.0) is (min_rating, max_rating)
    fn score_range(&self) -> (f64, f64);

    /// Return a list of fields required to insert a new user
    fn fields_for_users(&self) -> Vec<Field>;

    /// Return a list of fields required to insert a new item
    fn fields_for_items(&self) -> Vec<Field>;

    /// Insert a new user frow a prototype
    fn insert_user<'a>(&self, proto: HashMap<&'a str, Value>) -> Result<Self::User>;

    /// Insert a new item frow a prototype
    fn insert_item<'a>(&self, proto: HashMap<&'a str, Value>) -> Result<Self::Item>;

    /// Createa a rating in user for an item
    fn insert_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
        score: f64,
    ) -> Result<Self::Rating>;

    /// Remove a rating in user for an item
    fn remove_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
    ) -> Result<Self::Rating>;

    /// Update a rating in user for an item
    fn update_rating(
        &self,
        user_id: &eid!(Self::User),
        item_id: &eid!(Self::Item),
        score: f64,
    ) -> Result<Self::Rating>;
}
