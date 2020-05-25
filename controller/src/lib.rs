use anyhow::Error;
use std::collections::HashMap;

pub trait User<I: Item> {
    type Id;

    fn id(&self) -> Self::Id;
    fn name(&self) -> &str;
    fn ratings(&self) -> &HashMap<I::Id, f64>;
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub trait Item {
    type Id;

    fn id(&self) -> Self::Id;
    fn name(&self) -> &str;
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub trait Controller<U, I>
where
    I: Item,
    U: User<I>,
{
    fn with_url(url: &str) -> Self;
    fn user_by_id(&self, id: U::Id) -> Result<U, Error>;
    fn item_by_id(&self, id: I::Id) -> Result<I, Error>;
    fn user_by_name(&self, name: &str) -> Result<Vec<U>, Error>;
    fn item_by_name(&self, name: &str) -> Result<Vec<I>, Error>;
    fn all_users(&self) -> Result<Vec<U>, Error>;
    fn all_users_except(&self, id: U::Id) -> Result<Vec<U>, Error>;
}

pub mod error {
    use std::fmt::{Debug, Display};

    #[derive(Debug, thiserror::Error)]
    #[error("Entity with id({0}) not found")]
    pub struct NotFoundById<I: Debug + Display>(pub I);

    #[derive(Debug, thiserror::Error)]
    #[error("Entity with name({0}) not found")]
    pub struct NotFoundByName<I: Debug + Display>(pub I);
}
