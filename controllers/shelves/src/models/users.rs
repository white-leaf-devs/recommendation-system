use crate::schema::users;
use controller::Entity;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Default)]
pub struct User {
    pub id: i32
}

impl Entity for User {
    fn get_id(&self) -> String {
        self.id.to_string()
    }
}

// To insert a new user into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32
}
