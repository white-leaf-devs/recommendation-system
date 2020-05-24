use anyhow::Error;
use std::collections::HashMap;

pub trait User {
    fn id(&self) -> u64;
    fn name(&self) -> &str;
    fn ratings(&self) -> &HashMap<u64, f64>;
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub trait Item {
    fn id(&self) -> u64;
    fn name(&self) -> &str;
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub trait Controller<U, I>
where
    U: User,
    I: Item,
{
    fn with_url(url: &str) -> Self;
    fn user_by_id(&self, id: u64) -> Result<U, Error>;
    fn item_by_id(&self, id: u64) -> Result<I, Error>;
    fn user_by_name(&self, name: &str) -> Result<Vec<U>, Error>;
    fn item_by_name(&self, name: &str) -> Result<Vec<I>, Error>;
    fn all_users(&self) -> Result<Vec<U>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Entity with id({0}) not found")]
    NotFound(u64),

    #[error("Entity with name({0}) not found")]
    NotFoundByName(String),
}
