#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{
    movies::Movie,
    ratings::Rating,
    users::{Mean, User},
};
use crate::schema::{movies, ratings, users};
use anyhow::Error;
use controller::{error::ErrorKind, Controller, MapedRatings, Ratings, SearchBy};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use mongodb::bson::doc;
use mongodb::sync::{Client, Database};
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct SimpleMovieController {
    pg_conn: PgConnection,
    mongo_db: Database,
}

impl SimpleMovieController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url(
            "postgres://postgres:@localhost/simple-movie",
            "mongodb://localhost:27017",
            "simple-movie",
        )
    }

    pub fn with_url(psql_url: &str, mongo_url: &str, mongo_db: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(psql_url)?;
        let client = Client::with_uri_str(mongo_url)?;
        let mongo_db = client.database(mongo_db);

        Ok(Self { pg_conn, mongo_db })
    }
}

impl Controller<User, i32, Movie, i32> for SimpleMovieController {
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

    fn items(&self) -> Result<Vec<Movie>, Error> {
        let movies = movies::table.load::<Movie>(&self.pg_conn)?;
        Ok(movies)
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

    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Movie>, Error> {
        let items = movies::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Movie>(&self.pg_conn)?;

        Ok(items)
    }

    fn users_who_rated(&self, items: &[Movie]) -> Result<MapedRatings<i32, i32>, Error> {
        let collection = self.mongo_db.collection("users_who_rated");
        let ids: Vec<_> = items.iter().map(|m| m.id).collect();

        let cursor = collection.find(
            doc! {
                "item_id": { "$in": ids }
            },
            None,
        )?;

        let mut items_users = HashMap::new();
        for doc in cursor {
            let doc = doc?;
            let item_id = doc.get_i32("item_id")?;

            for (user_id, score) in doc.get_document("scores")? {
                let user_id: i32 = user_id.parse()?;
                let score = score.as_f64().ok_or_else(|| ErrorKind::BsonConvert)?;

                items_users
                    .entry(item_id)
                    .or_insert_with(HashMap::new)
                    .insert(user_id, score);
            }
        }

        Ok(items_users)
    }

    fn create_partial_users(&self, user_ids: &[i32]) -> Result<Vec<User>, Error> {
        user_ids
            .iter()
            .map(|id| -> Result<User, Error> {
                Ok(User {
                    id: *id,
                    ..Default::default()
                })
            })
            .collect()
    }

    fn create_partial_items(&self, item_ids: &[i32]) -> Result<Vec<Movie>, Error> {
        item_ids
            .iter()
            .map(|id| -> Result<Movie, Error> {
                Ok(Movie {
                    id: *id,
                    ..Default::default()
                })
            })
            .collect()
    }

    fn ratings_by(&self, user: &User) -> Result<Ratings<i32>, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.movie_id, rating.score))
            .collect();

        Ok(ratings)
    }

    fn maped_ratings(&self) -> Result<MapedRatings<i32, i32>, Error> {
        let ratings = ratings::table.load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.movie_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn maped_ratings_by(&self, users: &[User]) -> Result<MapedRatings<i32, i32>, Error> {
        let ratings = Rating::belonging_to(users).load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.movie_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn maped_ratings_except(&self, user: &User) -> Result<MapedRatings<i32, i32>, Error> {
        let ratings = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load::<Rating>(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            maped_ratings
                .entry(rating.user_id)
                .or_insert_with(HashMap::new)
                .insert(rating.movie_id, rating.score);
        }

        Ok(maped_ratings)
    }

    fn get_range(&self) -> (f64, f64) {
        (1., 5.)
    }
    fn get_means(&self, users: &[User]) -> HashMap<i32, f64> {
        let means = Mean::belonging_to(users)
            .load::<Mean>(&self.pg_conn)
            .unwrap();

        let mut means_by_user = HashMap::new();

        for mean in means {
            means_by_user.insert(mean.user_id, mean.val);
        }

        means_by_user
    }
}

#[cfg(feature = "test-controller")]
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use controller::Entity;

    #[test]
    fn query_user_by_id() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;

        let users = controller.users_by(&SearchBy::id("53"))?;
        assert_eq!(users[0].get_id(), 53);

        Ok(())
    }

    #[test]
    fn query_user_by_name() -> Result<(), Error> {
        let controller = SimpleMovieController::new()?;

        let users = controller.users_by(&SearchBy::name("Chris"))?;
        assert_eq!(users.len(), 2);

        Ok(())
    }
}
