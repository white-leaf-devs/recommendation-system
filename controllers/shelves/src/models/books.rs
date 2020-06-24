// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::schema::books;
use controller::Entity;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Default)]
pub struct Book {
    pub id: i32,
}

// To insert a new movie into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "books"]
pub struct NewBook {
    pub id: i32,
}

impl Entity for Book {
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}
