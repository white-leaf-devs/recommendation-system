#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{books::Book, ratings::Rating, users::User};
use crate::schema::{books, ratings, users};
use anyhow::Error;
use controller::{error::ErrorKind, Controller, MapedRatings, Ratings, SearchBy};
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
    fn users(&self, by: &SearchBy) -> Result<Vec<User>, Error> {
        match by {
            SearchBy::Id(id) => {
                let id: i32 = id.parse()?;
                let users = users::table.filter(users::id.eq(id)).load(&self.pg_conn)?;

                if users.is_empty() {
                    Err(ErrorKind::NotFoundById(id.to_string()).into())
                } else {
                    Ok(users)
                }
            }

            SearchBy::Name(name) => Err(ErrorKind::NotFoundByName(name.clone()).into()),
            SearchBy::Custom(k, v) => Err(ErrorKind::NotFoundByCustom(k.clone(), v.clone()).into()),
        }
    }

    fn items(&self, by: &SearchBy) -> Result<Vec<Book>, Error> {
        match by {
            SearchBy::Id(id) => {
                let books = books::table.filter(books::id.eq(id)).load(&self.pg_conn)?;

                if books.is_empty() {
                    Err(ErrorKind::NotFoundById(id.to_string()).into())
                } else {
                    Ok(books)
                }
            }

            SearchBy::Name(name) => {
                let books = books::table
                    .filter(books::title.eq(name))
                    .load(&self.pg_conn)?;

                if books.is_empty() {
                    Err(ErrorKind::NotFoundByName(name.clone()).into())
                } else {
                    Ok(books)
                }
            }

            SearchBy::Custom(k, v) => Err(ErrorKind::NotFoundByCustom(k.clone(), v.clone()).into()),
        }
    }

    fn ratings_by(&self, user: &User) -> Result<Ratings, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| (rating.book_id.clone(), rating.score))
            .collect();

        Ok(ratings)
    }

    fn ratings_except(&self, user: &User) -> Result<MapedRatings, Error> {
        let ratings: Vec<Rating> = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.to_string())
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
        let controller = BooksController::new()?;

        let users = controller.users(&SearchBy::id("2"))?;
        assert_eq!(users[0].get_id(), "2".to_string());

        Ok(())
    }

    #[test]
    fn query_item_by_name() -> Result<(), Error> {
        let controller = BooksController::new()?;

        let book = controller.items(&SearchBy::name("Jane Doe"))?;
        assert_eq!(book[0].get_id(), "1552041778".to_string());

        Ok(())
    }
}
