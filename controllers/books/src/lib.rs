#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{books::Book, ratings::Rating, users::User};
use crate::schema::{books, ratings, users};
use anyhow::Error;
use controller::{Controller, MapedRatings, Ratings};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> PgConnection {
    PgConnection::establish(&url).unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

pub struct BooksController {
    pg_conn: PgConnection,
}

impl Controller<User, Book> for BooksController {
    fn new() -> Self {
        Self::with_url("postgres://postgres:@localhost/books")
    }

    fn with_url(url: &str) -> Self {
        let pg_conn = establish_connection(url);
        Self { pg_conn }
    }

    fn user_by_id(&self, id: i32) -> Result<User, Error> {
        let user = users::table
            .filter(users::id.eq(id))
            .first::<User>(&self.pg_conn)?;

        Ok(user)
    }

    fn item_by_id(&self, id: String) -> Result<Book, Error> {
        let movie = books::table
            .filter(books::id.eq(id))
            .first::<Book>(&self.pg_conn)?;

        Ok(movie)
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<Book>, Error> {
        let books: Vec<Book> = books::table
            .filter(books::title.eq(name))
            .load(&self.pg_conn)?;

        Ok(books)
    }

    fn ratings_by_user(&self, user: &User) -> Result<Ratings<String>, Error> {
        let ratings: HashMap<_, _> = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| (rating.book_id.clone(), rating.score))
            .collect();

        Ok(ratings)
    }

    fn ratings_except_for(&self, user: &User) -> Result<MapedRatings<i32, String>, Error> {
        let ratings: Vec<Rating> = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.book_id.clone(), rating.score);
        }

        Ok(maped_ratings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use controller::Entity;

    #[test]
    fn query_user_by_id() -> Result<(), Error> {
        let controller = BooksController::new();

        let user = controller.user_by_id(2)?;
        assert_eq!(user.get_id(), 2);

        Ok(())
    }

    #[test]
    fn query_item_by_name() -> Result<(), Error> {
        let controller = BooksController::new();

        let book = controller.item_by_name("Jane Doe")?;
        assert_eq!(book[0].get_id(), "1552041778".to_string());

        Ok(())
    }
}
