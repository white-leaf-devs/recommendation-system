use anyhow::Error;
use std::{collections::HashMap, hash::Hash, ops::Deref};

use prettytable::{cell, format::consts::FORMAT_BOX_CHARS, row, table, Table};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Id(pub String);

impl<T: ToString> From<T> for Id {
    fn from(t: T) -> Self {
        Id(t.to_string())
    }
}

impl Deref for Id {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub type Ratings = HashMap<Id, f64>;
pub type MapedRatings = HashMap<Id, Ratings>;

pub fn ratings_to_table(map: &Ratings) -> Table {
    let mut table = Table::new();
    for (key, val) in map {
        table.add_row(row![key.0, val]);
    }

    table.set_format(*FORMAT_BOX_CHARS);
    table
}

pub trait Entity {
    fn get_id(&self) -> Id;

    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
    }

    fn to_table(&self) -> Table {
        let mut table = table![["id", self.get_id().0]];
        for (key, val) in self.get_data() {
            table.add_row(row![key, val]);
        }

        table.set_format(*FORMAT_BOX_CHARS);
        table
    }
}

pub trait Controller<U, I> {
    fn user_by_id(&self, id: &Id) -> Result<U>;
    fn item_by_id(&self, id: &Id) -> Result<I>;
    fn ratings_by_user(&self, user: &U) -> Result<Ratings>;
    fn ratings_except_for(&self, user: &U) -> Result<MapedRatings>;

    fn user_by_name(&self, name: &str) -> Result<Vec<U>> {
        Err(error::NotFoundByName(name.into()).into())
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<I>> {
        Err(error::NotFoundByName(name.into()).into())
    }
}

pub mod error {
    use std::fmt::Debug;

    #[derive(Debug, thiserror::Error)]
    #[error("Entity with name({0}) not found")]
    pub struct NotFoundByName(pub String);
}
