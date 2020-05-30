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

pub struct SimpleMovieController {
    pg_conn: PgConnection,
}

impl SimpleMovieController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url("postgres://postgres:@localhost/simple-movie")
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(url)?;
        Ok(Self { pg_conn })
    }
}

impl Controller<User, Movie> for SimpleMovieController {
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

            SearchBy::Name(name) => {
                let users = users::table
                    .filter(users::name.eq(name))
                    .load(&self.pg_conn)?;

                if users.is_empty() {
                    Err(ErrorKind::NotFoundByName(name.clone()).into())
                } else {
                    Ok(users)
                }
            }

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

    fn items(&self, by: &SearchBy) -> Result<Vec<Movie>, Error> {
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
                    .filter(movies::name.eq(name))
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

    fn ratings_by(&self, user: &User) -> Result<Ratings, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.movie_id.to_string(), rating.score))
            .collect();

        Ok(ratings)
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
    use controller::Entity;

    #[test]
    fn query_user_by_id() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;

        let users = controller.users(&SearchBy::id("53"))?;
        assert_eq!(users[0].get_id(), "53".to_string());

        Ok(())
    }

    #[test]
    fn query_user_by_name() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;

        let users = controller.users(&SearchBy::name("Chris"))?;
        assert_eq!(users.len(), 2);

        Ok(())
    }
}
