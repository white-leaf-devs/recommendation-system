use crate::schema::*;
use controller::{Item, User as UserTrait};
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
}

// To insert a new user into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
}

// To the controller, this users include their ratings
#[derive(Debug, Clone)]
pub struct CompleteUser {
    pub inner: User,
    pub ratings: HashMap<u64, f64>,
}

impl UserTrait for CompleteUser {
    fn id(&self) -> u64 {
        self.inner.id as u64
    }

    fn name(&self) -> &str {
        &self.inner.name
    }

    fn ratings(&self) -> &HashMap<u64, f64> {
        &self.ratings
    }
}

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct Movie {
    pub id: i32,
    pub name: String,
}

// To insert a new movie into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "movies"]
pub struct NewMovie<'a> {
    pub name: &'a str,
}

impl Item for Movie {
    fn id(&self) -> u64 {
        self.id as u64
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub movie_id: i32,
    pub score: f64,
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating {
    pub user_id: i32,
    pub movie_id: i32,
    pub score: f64,
}
