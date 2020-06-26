// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub mod entity;
pub mod error;
pub mod lazy;
pub mod searchby;

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

pub type Result<T> = std::result::Result<T, Error>;
pub type Ratings<ItemId, Value = f64> = HashMap<ItemId, Value>;
pub type MapedRatings<UserId, ItemId, Value = f64> = HashMap<UserId, Ratings<ItemId, Value>>;

pub trait Controller<User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
{
    fn users(&self) -> Result<Vec<User>>;
    fn users_by(&self, by: &SearchBy) -> Result<Vec<User>>;
    fn users_by_chunks(&self, chunk_size: usize) -> LazyUserChunks<User, UserId, Item, ItemId>
    where
        Self: Sized,
    {
        LazyUserChunks {
            curr_offset: 0,
            chunk_size,
            controller: self,
        }
    }

    #[allow(unused_variables)]
    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<User>> {
        Err(error::ErrorKind::NotImplemented.into())
    }

    fn items(&self) -> Result<Vec<Item>>;
    fn items_by(&self, by: &SearchBy) -> Result<Vec<Item>>;
    fn items_by_chunks(&self, chunk_size: usize) -> LazyItemChunks<User, UserId, Item, ItemId>
    where
        Self: Sized,
    {
        LazyItemChunks {
            curr_offset: 0,
            chunk_size,
            controller: self,
        }
    }

    #[allow(unused_variables)]
    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Item>> {
        Err(error::ErrorKind::NotImplemented.into())
    }

    fn create_partial_users(&self, user_ids: &[UserId]) -> Result<Vec<User>>;
    fn create_partial_items(&self, item_ids: &[ItemId]) -> Result<Vec<Item>>;

    fn users_who_rated(&self, items: &[Item]) -> Result<MapedRatings<ItemId, UserId>>;
    fn ratings_by(&self, user: &User) -> Result<Ratings<ItemId>>;
    fn maped_ratings(&self) -> Result<MapedRatings<UserId, ItemId>>;
    fn maped_ratings_by(&self, users: &[User]) -> Result<MapedRatings<UserId, ItemId>>;
    fn maped_ratings_except(&self, user: &User) -> Result<MapedRatings<UserId, ItemId>>;
    fn means_for(&self, users: &[User]) -> Result<HashMap<UserId, f64>>;

    fn score_range(&self) -> (f64, f64);
}
