use crate::schema::users;
use controller::Entity;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
}

// To insert a new user into the database
impl Entity for User {
    fn get_id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
}
