// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::schema::{means, users};
use controller::Entity;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
}

// To insert a new user into the database
impl Entity for User {
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
}

#[derive(Debug, Clone, Identifiable, Queryable, Associations)]
#[primary_key(user_id)]
#[belongs_to(User)]
pub struct Mean {
    pub user_id: i32,
    pub val: f64,
    pub score_number: i32,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "means"]
pub struct NewMean {
    pub user_id: i32,
    pub val: f64,
    pub score_number: i32,
}
