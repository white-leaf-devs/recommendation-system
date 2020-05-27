use crate::schema::movies;
use common_macros::hash_map;
use controller::Entity;
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct Movie {
    pub id: i32,
    pub name: String,
}

impl Entity for Movie {
    fn get_id(&self) -> String {
        self.id.to_string()
    }

    fn get_data(&self) -> HashMap<String, String> {
        hash_map! {
            "name".into() => self.name.clone()
        }
    }
}

// To insert a new movie into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "movies"]
pub struct NewMovie<'a> {
    pub name: &'a str,
}
