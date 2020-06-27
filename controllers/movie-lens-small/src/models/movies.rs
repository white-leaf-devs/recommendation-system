// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::schema::movies;
use common_macros::hash_map;
use controller::Entity;
use std::collections::HashMap;

#[derive(Debug, Clone, Identifiable, Queryable, Default)]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub genres: String,
}

impl Entity for Movie {
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }

    fn get_data(&self) -> HashMap<String, String> {
        hash_map! {
            "title".into() => self.title.clone(),
            "genres".into() => self.genres.clone(),
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "movies"]
pub struct NewMovie<'a> {
    pub id: i32,
    pub title: &'a str,
    pub genres: &'a str,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "movies"]
pub struct NewUnseenMovie<'a> {
    pub title: &'a str,
    pub genres: &'a str,
}
