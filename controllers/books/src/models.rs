use crate::schema::*;
use controller::{Item, User as UserT};
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub year: i16,
    pub publisher: String,
}

// To insert a new movie into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "books"]
pub struct NewBook<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub author: &'a str,
    pub year: i16,
    pub publisher: &'a str,
}

impl Item for Book {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn name(&self) -> Option<&str> {
        Some(&self.title)
    }

    fn metadata(&self) -> HashMap<String, String> {
        [
            ("title".into(), self.title.to_string()),
            ("author".into(), self.author.to_string()),
            ("year".into(), self.year.to_string()),
            ("publisher".into(), self.publisher.to_string()),
        ]
        .iter()
        .cloned()
        .collect()
    }
}

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub id: i32,
    pub location: String,
    pub age: Option<i16>,
}

// To insert a new user into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: i32,
    pub location: &'a str,
    pub age: Option<i16>,
}

// To the controller, this users include their ratings
#[derive(Debug, Clone)]
pub struct CompleteUser {
    pub inner: User,
    pub ratings: HashMap<String, f64>,
}

impl UserT<Book> for CompleteUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.inner.id
    }

    fn name(&self) -> Option<&str> {
        None
    }

    fn ratings(&self) -> &HashMap<<Book as Item>::Id, f64> {
        &self.ratings
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("location".into(), self.inner.location.to_string());

        if let Some(age) = &self.inner.age {
            data.insert("age".into(), age.to_string());
        }

        data
    }
}

// To query data from the database
#[derive(Debug, Clone, Queryable)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub book_id: String,
    pub score: f64,
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating<'a> {
    pub user_id: i32,
    pub book_id: &'a str,
    pub score: f64,
}
