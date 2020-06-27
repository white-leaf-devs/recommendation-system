// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::movies::Movie;
use super::users::User;
use crate::schema::ratings;
use common_macros::hash_map;
use controller::Entity;
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Movie)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub movie_id: i32,
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
            "movie_id".into() => self.movie_id.to_string(),
            "score".into() => self.score.to_string(),
        }
    }
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating {
    pub user_id: i32,
    pub movie_id: i32,
    pub score: f64,
}
