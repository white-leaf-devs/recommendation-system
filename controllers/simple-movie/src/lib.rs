#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{movies::Movie, ratings::Rating, users::User};
use crate::schema::{movies, ratings, users};
use anyhow::Error;
use controller::{Controller, MapedRatings, Ratings};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> PgConnection {
    PgConnection::establish(&url).unwrap_or_else(|_| panic!("Error connecting to {}", url))
}
pub struct SimpleMovieController {
    pg_conn: PgConnection,
}

impl Controller<User, Movie> for SimpleMovieController {
    fn new() -> Self {
        Self::with_url("postgres://postgres:@localhost/simple-movie")
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

    fn item_by_id(&self, id: i32) -> Result<Movie, Error> {
        let movie = movies::table
            .filter(movies::id.eq(id))
            .first::<Movie>(&self.pg_conn)?;

        Ok(movie)
    }

    fn user_by_name(&self, name: &str) -> Result<Vec<User>, Error> {
        let users: Vec<User> = users::table
            .filter(users::name.eq(name))
            .load(&self.pg_conn)?;

        Ok(users)
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<Movie>, Error> {
        let movies: Vec<Movie> = movies::table
            .filter(movies::name.eq(name))
            .load(&self.pg_conn)?;

        Ok(movies)
    }

    fn ratings_by_user(&self, user: &User) -> Result<Ratings<i32>, Error> {
        let ratings: HashMap<_, _> = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| (rating.movie_id, rating.score))
            .collect();

        Ok(ratings)
    }

    fn ratings_except_for(&self, user: &User) -> Result<MapedRatings<i32, i32>, Error> {
        let ratings: Vec<Rating> = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.movie_id, rating.score);
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
        let controller = SimpleMovieController::new();

        let user = controller.user_by_id(53)?;
        assert_eq!(user.get_id(), 53);

        Ok(())
    }

    #[test]
    fn query_user_by_name() -> Result<(), Error> {
        let controller = SimpleMovieController::new();

        let users = controller.user_by_name("Chris")?;
        assert_eq!(users.len(), 2);

        Ok(())
    }
}
