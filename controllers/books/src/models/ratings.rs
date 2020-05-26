use super::users::User;
use crate::schema::ratings;

// To query data from the database
#[derive(Debug, Clone, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
pub struct Rating {
    pub id: i32,
    pub user_id: i32,
    pub book_id: String,
    pub score: f64,
}

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating<'a> {
    pub user_id: i32,
    pub book_id: &'a str,
    pub score: f64,
}
