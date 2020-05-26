use crate::schema::users;
use common_macros::hash_map;
use controller::{Entity, Id};
use std::collections::HashMap;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
}

// To insert a new user into the database
impl Entity for User {
    fn get_id(&self) -> Id {
        self.id.into()
    }

    fn get_data(&self) -> HashMap<String, String> {
        hash_map! {
            "name".into() => self.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
}
