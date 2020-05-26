use crate::schema::users;
use controller::{Entity, Id};

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
}

// To insert a new user into the database
impl Entity for User {
    fn get_id(&self) -> Id {
        self.id.into()
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
}
