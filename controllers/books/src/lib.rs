#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{books::Book, ratings::Rating, users::User};
use crate::schema::{books, ratings, users};
use anyhow::Error;
use controller::{Controller, Id, MapedRatings, Ratings};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct BooksController {
    pg_conn: PgConnection,
}

impl BooksController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url("postgres://postgres:@localhost/books")
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(url)?;
        Ok(Self { pg_conn })
    }
}

impl Controller<User, Book> for BooksController {
    fn user_by_id(&self, id: &Id) -> Result<User, Error> {
        let id: i32 = id.parse()?;

        let user = users::table
            .filter(users::id.eq(id))
            .first::<User>(&self.pg_conn)?;

        Ok(user)
    }

    fn item_by_id(&self, id: &Id) -> Result<Book, Error> {
        let id = id.to_string();

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

    fn ratings_by_user(&self, user: &User) -> Result<Ratings, Error> {
        let ratings: HashMap<_, _> = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| ((&rating.book_id).into(), rating.score))
            .collect();

        Ok(ratings)
    }

    fn ratings_except_for(&self, user: &User) -> Result<MapedRatings, Error> {
        let ratings: Vec<Rating> = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.into())
                .or_insert_with(HashMap::new)
                .insert(rating.book_id.into(), rating.score);
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
        let controller = BooksController::new()?;

        let user = controller.user_by_id(&2.into())?;
        assert_eq!(user.get_id(), 2.into());

        Ok(())
    }

    #[test]
    fn query_item_by_name() -> Result<(), Error> {
        let controller = BooksController::new()?;

        let book = controller.item_by_name("Jane Doe")?;
        assert_eq!(book[0].get_id(), "1552041778".into());

        Ok(())
    }
}
