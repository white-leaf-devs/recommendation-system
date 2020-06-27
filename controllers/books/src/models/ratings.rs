// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::books::Book;
use super::users::User;
use crate::schema::ratings;
use common_macros::hash_map;
use controller::Entity;
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Book)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub book_id: String,
    pub score: f64,
}

impl Entity for Rating {
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }

    fn get_data(&self) -> HashMap<String, String> {
        hash_map! {
            "user_id".into() => self.user_id.to_string(),
            "book_id".into() => self.book_id.clone(),
            "score".into() => self.score.to_string(),
        }
    }
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating<'a> {
    pub user_id: i32,
    pub book_id: &'a str,
    pub score: f64,
}
