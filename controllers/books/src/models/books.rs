use crate::schema::books;
use common_macros::hash_map;
use controller::Entity;
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Default)]
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

impl Entity for Book {
    type Id = String;
    fn get_id(&self) -> Self::Id {
        self.id.clone()
    }

    fn get_data(&self) -> HashMap<String, String> {
        hash_map! {
            "title".into() => self.title.clone(),
            "author".into() => self.author.clone(),
            "year".into() => self.year.to_string(),
            "publisher".into() => self.publisher.clone()
        }
    }
}
