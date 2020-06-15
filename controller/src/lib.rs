use anyhow::Error;
use prettytable::{cell, format::consts::FORMAT_NO_LINESEP, row, table, Table};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SearchBy {
    Id(String),
    Name(String),
    Custom(String, String),
}

impl SearchBy {
    pub fn id(id: &str) -> Self {
        Self::Id(id.into())
    }

    pub fn name(name: &str) -> Self {
        Self::Name(name.into())
    }

    pub fn custom(key: &str, val: &str) -> Self {
        Self::Custom(key.into(), val.into())
    }
}

impl Display for SearchBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchBy::Id(id) => write!(f, "id({})", id),
            SearchBy::Name(name) => write!(f, "name({})", name),
            SearchBy::Custom(key, val) => write!(f, "{}({})", key, val),
        }
    }
}

pub trait Entity {
    type Id;

    fn get_id(&self) -> Self::Id;
    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
    }
}

pub trait ToTable {
    fn to_table(&self) -> Table;
}

impl<I: ToString, E: Entity<Id = I>> ToTable for E {
    fn to_table(&self) -> Table {
        let mut table = table![["id", self.get_id()]];

        for (key, val) in self.get_data() {
            table.add_row(row![key, val]);
        }

        table.set_format(*FORMAT_NO_LINESEP);
        table
    }
}

impl<K, V, B> ToTable for HashMap<K, V, B>
where
    K: ToString,
    V: ToString,
{
    fn to_table(&self) -> Table {
        let mut table = Table::new();

        for (key, val) in self {
            table.add_row(row![key, val]);
        }

        table.set_format(*FORMAT_NO_LINESEP);
        table
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub type Ratings<ItemId, Value = f64> = HashMap<ItemId, Value>;
pub type MapedRatings<UserId, ItemId, Value = f64> = HashMap<UserId, Ratings<ItemId, Value>>;

pub struct LazyUserChunks<'a, User, UserId, Item, ItemId> {
    curr_offset: usize,
    chunk_size: usize,
    controller: &'a dyn Controller<User, UserId, Item, ItemId>,
}

impl<'a, User, UserId, Item, ItemId> Iterator for LazyUserChunks<'a, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
{
    type Item = Vec<User>;

    fn next(&mut self) -> Option<Self::Item> {
        let users = self
            .controller
            .users_offset_limit(self.curr_offset, self.chunk_size)
            .ok();

        self.curr_offset += self.chunk_size;
        match users {
            Some(users) => {
                if users.is_empty() {
                    None
                } else {
                    Some(users)
                }
            }
            None => None,
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.curr_offset = n * self.chunk_size;
        self.next()
    }
}

pub struct LazyItemChunks<'a, User, UserId, Item, ItemId> {
    curr_offset: usize,
    chunk_size: usize,
    controller: &'a dyn Controller<User, UserId, Item, ItemId>,
}

impl<'a, User, UserId, Item, ItemId> Iterator for LazyItemChunks<'a, User, UserId, Item, ItemId>
where
    User: Entity<Id = UserId>,
    Item: Entity<Id = ItemId>,
{
    type Item = Vec<Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = self
            .controller
            .items_offset_limit(self.curr_offset, self.chunk_size)
            .ok();

        self.curr_offset += self.chunk_size;
        match items {
            Some(items) => {
                if items.is_empty() {
                    None
                } else {
                    Some(items)
                }
            }
            None => None,
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.curr_offset = n * self.chunk_size;
        self.next()
    }
}

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

    fn users_who_rated(&self, items: &[Item]) -> Result<MapedRatings<ItemId, UserId>>;
    fn ratings_by(&self, user: &User) -> Result<Ratings<ItemId>>;
    fn maped_ratings(&self) -> Result<MapedRatings<UserId, ItemId>>;
    fn maped_ratings_by(&self, users: &[User]) -> Result<MapedRatings<UserId, ItemId>>;
    fn maped_ratings_except(&self, user: &User) -> Result<MapedRatings<UserId, ItemId>>;

    fn get_range(&self) -> (f64, f64);
    fn get_means(&self, users: &Vec<User>) -> HashMap<UserId, f64>;
}

pub mod error {
    use thiserror::Error as DError;

    #[derive(Debug, Clone, DError)]
    pub enum ErrorKind {
        #[error("Couldn't found entity with id({0})")]
        NotFoundById(String),

        #[error("Couldn't found entity with name({0})")]
        NotFoundByName(String),

        #[error("Couldn't found entity with {0}({1})")]
        NotFoundByCustom(String, String),

        #[error("Controller function not implemented")]
        NotImplemented,
    }
}
