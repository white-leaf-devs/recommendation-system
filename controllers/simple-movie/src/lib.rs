#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{CompleteUser, Movie, Rating, User};
use crate::schema::{movies, ratings, users};
use anyhow::Error;
use controller::{error, Controller};
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn establish_connection(url: &str) -> PgConnection {
    PgConnection::establish(&url).unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

pub struct SimpleMovieController {
    pg_conn: PgConnection,
}

impl Controller<CompleteUser, Movie> for SimpleMovieController {
    fn with_url(url: &str) -> Self {
        let pg_conn = establish_connection(url);
        SimpleMovieController { pg_conn }
    }

    fn user_by_id(&self, id: i32) -> Result<CompleteUser, Error> {
        let user = users::table
            .filter(users::id.eq(id))
            .limit(1)
            .load::<User>(&self.pg_conn)?
            .get(0)
            .cloned()
            .ok_or_else(|| error::NotFoundById(id))?;

        let ratings = ratings::table
            .filter(ratings::user_id.eq(id))
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| (rating.movie_id, rating.score))
            .collect();

        Ok(CompleteUser {
            inner: user,
            ratings,
        })
    }

    fn item_by_id(&self, id: i32) -> Result<Movie, Error> {
        movies::table
            .filter(movies::id.eq(id))
            .limit(1)
            .load(&self.pg_conn)?
            .get(0)
            .cloned()
            .ok_or_else(|| error::NotFoundById(id).into())
    }

    fn user_by_name(&self, name: &str) -> Result<Vec<CompleteUser>, Error> {
        let users: Vec<_> = users::table
            .filter(users::name.eq(name))
            .load::<User>(&self.pg_conn)?;

        let mut complete_users = Vec::new();
        for user in users {
            let ratings = ratings::table
                .filter(ratings::user_id.eq(user.id))
                .load::<Rating>(&self.pg_conn)?
                .iter()
                .map(|rating| (rating.movie_id, rating.score))
                .collect();

            complete_users.push(CompleteUser {
                inner: user,
                ratings,
            });
        }

        Ok(complete_users)
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<Movie>, Error> {
        let movies: Vec<Movie> = movies::table
            .filter(movies::name.eq(name))
            .load(&self.pg_conn)?;

        Ok(movies)
    }

    fn all_users(&self) -> Result<Vec<CompleteUser>, Error> {
        let users = users::table.load::<User>(&self.pg_conn)?;

        let mut complete_users = Vec::new();
        for user in users {
            let ratings = ratings::table
                .filter(ratings::user_id.eq(user.id))
                .load::<Rating>(&self.pg_conn)?
                .iter()
                .map(|rating| (rating.movie_id, rating.score))
                .collect();

            complete_users.push(CompleteUser {
                inner: user,
                ratings,
            });
        }

        Ok(complete_users)
    }

    fn all_users_except(&self, id: i32) -> Result<Vec<CompleteUser>, Error> {
        let users = users::table
            .filter(users::id.is_distinct_from(id))
            .load::<User>(&self.pg_conn)?;

        let mut complete_users = Vec::new();
        for user in users {
            let ratings = ratings::table
                .filter(ratings::user_id.eq(user.id))
                .load::<Rating>(&self.pg_conn)?
                .iter()
                .map(|rating| (rating.movie_id, rating.score))
                .collect();

            complete_users.push(CompleteUser {
                inner: user,
                ratings,
            });
        }

        Ok(complete_users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use controller::User;

    #[test]
    fn query_user_by_id() -> Result<(), Error> {
        let controller =
            SimpleMovieController::with_url("postgres://postgres:@localhost/simple-movie");

        let user = controller.user_by_id(53)?;
        assert_eq!(user.id(), 53);

        Ok(())
    }

    #[test]
    fn query_user_by_name() -> Result<(), Error> {
        let controller =
            SimpleMovieController::with_url("postgres://postgres:@localhost/simple-movie");

        let users = controller.user_by_name("Chris")?;
        assert_eq!(users.len(), 2);
        for user in users {
            assert_eq!("Chris", user.name());
        }

        Ok(())
    }
}
