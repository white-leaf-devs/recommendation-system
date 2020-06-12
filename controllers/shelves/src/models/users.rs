use crate::schema::users;
use controller::Entity;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Default)]
pub struct User {
    pub id: i32,
}

impl Entity for User {
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}

// To insert a new user into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
}