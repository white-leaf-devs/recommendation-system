use super::movies::Movie;
use super::users::User;
use crate::schema::ratings;

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

// To insert a new rating into the database
#[derive(Debug, Clone, Insertable)]
#[table_name = "ratings"]
pub struct NewRating {
    pub user_id: i32,
    pub movie_id: i32,
    pub score: f64,
}
