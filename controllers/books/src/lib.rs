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
    fn users(&self) -> Result<Vec<User>, Error> {
        let users = users::table.load::<User>(&self.pg_conn)?;
        Ok(users)
    }

    fn users_by(&self, by: &SearchBy) -> Result<Vec<User>, Error> {
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

    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<User>, Error> {
        let users = users::table
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<User>(&self.pg_conn)?;

        Ok(users)
    }

    fn items(&self) -> Result<Vec<Book>, Error> {
        let items = books::table.load::<Book>(&self.pg_conn)?;
        Ok(items)
    }

    fn items_by(&self, by: &SearchBy) -> Result<Vec<Book>, Error> {
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

    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Book>, Error> {
        let items = books::table
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<Book>(&self.pg_conn)?;

        Ok(items)
    }

    fn ratings_by(&self, user: &User) -> Result<Ratings, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.book_id, rating.score))
            .collect();

        Ok(ratings)
    }

    fn maped_ratings(&self) -> Result<MapedRatings, Error> {
        let ratings = ratings::table.load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn maped_ratings_by(&self, users: &[User]) -> Result<MapedRatings, Error> {
        let ratings = Rating::belonging_to(users).load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn maped_ratings_except(&self, user: &User) -> Result<MapedRatings, Error> {
        let ratings = ratings::table
            .filter(ratings::user_id.ne(user.id))
            .load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(rating.book_id, rating.score);
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

        let users = controller.users_by(&SearchBy::id("2"))?;
        assert_eq!(users[0].get_id(), "2".to_string());

        Ok(())
    }

    #[test]
    fn query_item_by_name() -> Result<(), Error> {
        let controller = BooksController::new()?;

        let book = controller.items_by(&SearchBy::name("Jane Doe"))?;
        assert_eq!(book[0].get_id(), "1552041778".to_string());

        Ok(())
    }

    #[test]
    fn chunked_users() -> Result<(), Error> {
        let controller = BooksController::new()?;
        let mut chunk_iter = controller.users_by_chunks(80000);

        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(80000, chunk_iter.next().unwrap().len());
        assert_eq!(38858, chunk_iter.next().unwrap().len());
        assert!(chunk_iter.next().is_none());

        Ok(())
    }
}
