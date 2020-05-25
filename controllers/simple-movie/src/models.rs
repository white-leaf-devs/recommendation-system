use crate::schema::*;
use controller::{Item, User as UserT};
use std::collections::HashMap;

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
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
}

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
    pub ratings: HashMap<i32, f64>,
}

impl UserT<Movie> for CompleteUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.inner.id
    }

    fn name(&self) -> Option<&str> {
        Some(&self.inner.name)
    }

    fn ratings(&self) -> &HashMap<<Movie as Item>::Id, f64> {
        &self.ratings
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
