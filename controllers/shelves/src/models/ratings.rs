// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::books::Book;
use super::users::User;
use crate::schema::ratings;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Book)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub score: f64,
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating {
    pub user_id: i32,
    pub book_id: i32,
    pub score: f64,
}
