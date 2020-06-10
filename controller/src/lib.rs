use anyhow::Error;
use prettytable::{cell, format::consts::FORMAT_NO_LINESEP, row, table, Table};
use std::collections::HashMap;
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
    fn get_id(&self) -> String;
    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
    }
}

pub trait ToTable {
    fn to_table(&self) -> Table;
}

impl<E: Entity> ToTable for E {
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
pub type Ratings = HashMap<String, f64>;
pub type MapedRatings = HashMap<String, Ratings>;

pub struct LazyUserChunks<'a, U, I> {
    curr_offset: usize,
    chunk_size: usize,
    controller: &'a dyn Controller<U, I>,
}

impl<'a, U, I> Iterator for LazyUserChunks<'a, U, I> {
    type Item = Vec<U>;

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
}

pub struct LazyItemChunks<'a, U, I> {
    curr_offset: usize,
    chunk_size: usize,
    controller: &'a dyn Controller<U, I>,
}

impl<'a, U, I> Iterator for LazyItemChunks<'a, U, I> {
    type Item = Vec<I>;

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
}

pub trait Controller<U, I> {
    fn users(&self) -> Result<Vec<U>>;
    fn users_by(&self, by: &SearchBy) -> Result<Vec<U>>;
    fn users_by_chunks(&self, chunk_size: usize) -> LazyUserChunks<U, I>
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
    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<U>> {
        Err(error::ErrorKind::NotImplemented.into())
    }

    fn items(&self) -> Result<Vec<I>>;
    fn items_by(&self, by: &SearchBy) -> Result<Vec<I>>;
    fn items_by_chunks(&self, chunk_size: usize) -> LazyItemChunks<U, I>
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
    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<I>> {
        Err(error::ErrorKind::NotImplemented.into())
    }

    fn ratings_by(&self, user: &U) -> Result<Ratings>;
    fn maped_ratings(&self) -> Result<MapedRatings>;
    fn maped_ratings_by(&self, users: &[U]) -> Result<MapedRatings>;
    fn maped_ratings_except(&self, user: &U) -> Result<MapedRatings>;
}

pub mod error {
    use std::fmt::Debug;
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
