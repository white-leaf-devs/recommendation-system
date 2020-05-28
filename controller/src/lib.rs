use anyhow::Error;
use prettytable::{cell, format::consts::FORMAT_NO_LINESEP, row, table, Table};
use std::collections::HashMap;

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

//impl<K, V, B> ToTable for &'_ HashMap<K, V, B>
//where
//K: ToString,
//V: ToString,
//{
//fn to_table(&self) -> Table {
//let mut table = Table::new();

//for (key, val) in *self {
//table.add_row(row![key, val]);
//}

//table.set_format(*FORMAT_BOX_CHARS);
//table
//}
//}

pub type Result<T> = std::result::Result<T, Error>;
pub type Ratings = HashMap<String, f64>;
pub type MapedRatings = HashMap<String, Ratings>;

pub trait Controller<U, I> {
    fn users(&self, by: &SearchBy) -> Result<Vec<U>>;
    fn items(&self, by: &SearchBy) -> Result<Vec<I>>;
    fn ratings_by(&self, user: &U) -> Result<Ratings>;
    fn ratings_except(&self, user: &U) -> Result<MapedRatings>;
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
    }
}
