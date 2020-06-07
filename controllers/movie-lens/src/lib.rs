#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{movies::Movie, ratings::Rating, users::User};
use crate::schema::{movies, ratings, users};
use anyhow::Error;
use controller::{error::ErrorKind, Controller, MapedRatings, Ratings, SearchBy};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct MovieLensController {
    pg_conn: PgConnection,
}

impl MovieLensController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url("postgres://postgres:@localhost/movie-lens")
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(url)?;
        Ok(Self { pg_conn })
    }
}

impl Controller<User, Movie> for MovieLensController {
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
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<User>(&self.pg_conn)?;

        Ok(users)
    }

    fn items(&self) -> Result<Vec<Movie>, Error> {
        let items = movies::table.load::<Movie>(&self.pg_conn)?;
        Ok(items)
    }

    fn items_by(&self, by: &SearchBy) -> Result<Vec<Movie>, Error> {
        match by {
            SearchBy::Id(id) => {
                let id: i32 = id.parse()?;
                let movies = movies::table
                    .filter(movies::id.eq(id))
                    .load(&self.pg_conn)?;

                if movies.is_empty() {
                    Err(ErrorKind::NotFoundById(id.to_string()).into())
                } else {
                    Ok(movies)
                }
            }

            SearchBy::Name(name) => {
                let movies = movies::table
                    .filter(movies::title.eq(name))
                    .load(&self.pg_conn)?;

                if movies.is_empty() {
                    Err(ErrorKind::NotFoundByName(name.clone()).into())
                } else {
                    Ok(movies)
                }
            }

            SearchBy::Custom(k, v) => Err(ErrorKind::NotFoundByCustom(k.clone(), v.clone()).into()),
        }
    }

    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Movie>, Error> {
        let items = movies::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Movie>(&self.pg_conn)?;

        Ok(items)
    }

    fn ratings_by(&self, user: &User) -> Result<Ratings, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.movie_id.to_string(), rating.score))
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
                .insert(rating.movie_id.to_string(), rating.score);
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
                .insert(rating.movie_id.to_string(), rating.score);
        }

        Ok(maped_ratings)
    }

    fn maped_ratings_except(&self, user: &User) -> Result<MapedRatings, Error> {
        let ratings = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(rating.movie_id.to_string(), rating.score);
        }

        Ok(maped_ratings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;

    #[test]
    fn chunked_users() -> Result<(), Error> {
        let controller = MovieLensController::new()?;
        let mut lazy_iter = controller.users_by_chunks(64);

        assert_eq!(64, lazy_iter.next().unwrap().len());
        assert_eq!(64, lazy_iter.next().unwrap().len());

        Ok(())
    }

    #[test]
    fn maped_ratings_with_users_chunks() -> Result<(), Error> {
        let controller = MovieLensController::new()?;
        let mut lazy_iter = controller.users_by_chunks(64);

        assert_eq!(
            64,
            controller
                .maped_ratings_by(&lazy_iter.next().unwrap())?
                .len()
        );

        assert_eq!(
            64,
            controller
                .maped_ratings_by(&lazy_iter.next().unwrap())?
                .len()
        );

        Ok(())
    }
}