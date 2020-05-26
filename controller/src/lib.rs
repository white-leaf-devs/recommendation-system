use anyhow::Error;
use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher},
    ops::Deref,
};

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
pub type Ratings = HashMap<u64, f64>;
pub type MapedRatings = HashMap<Id, Ratings>;

pub fn make_hash<H: BuildHasher, K: Hash>(hasher: &H, key: K) -> u64 {
    let mut state = hasher.build_hasher();
    key.hash(&mut state);
    state.finish()
}

pub trait Entity {
    fn get_id(&self) -> Id;
    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
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
