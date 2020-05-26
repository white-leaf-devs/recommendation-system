use anyhow::Error;
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;
pub type Rating<K> = HashMap<K, f64>;
pub type MapedRatings<U, K> = HashMap<U, Rating<K>>;

pub trait Entity {
    type Id;

    fn get_id(&self) -> Self::Id;
    fn get_data(&self) -> HashMap<String, String> {
        Default::default()
    }
}

pub trait Controller<U, I>
where
    U: Entity,
    I: Entity,
{
    fn with_url(url: &str) -> Self;
    fn user_by_id(&self, id: U::Id) -> Result<U>;
    fn item_by_id(&self, id: I::Id) -> Result<I>;
    fn rating_by_user(&self, user: &U) -> Result<Rating<I::Id>>;
    fn all_ratings(&self) -> Result<MapedRatings<U::Id, I::Id>>;

    fn user_by_name(&self, name: &str) -> Result<Vec<U>> {
        Err(error::NotFoundByName(name.into()).into())
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<I>> {
        Err(error::NotFoundByName(name.into()).into())
    }
}

pub mod error {
    use std::fmt::{Debug, Display};

    #[derive(Debug, thiserror::Error)]
    #[error("Entity with id({0}) not found")]
    pub struct NotFoundById<I: Debug + Display>(pub I);

    #[derive(Debug, thiserror::Error)]
    #[error("Entity with name({0}) not found")]
    pub struct NotFoundByName(pub String);
}
