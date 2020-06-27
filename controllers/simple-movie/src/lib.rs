// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

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
use controller::{
    eid, error::ErrorKind, maped_ratings, means, ratings, Controller, Field, SearchBy, Type, Value,
};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use models::{movies::NewMovie, users::NewUser};
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

impl Controller for SimpleMovieController {
    type User = User;
    type Item = Movie;
    type Rating = Rating;

    fn users(&self) -> Result<Vec<Self::User>, Error> {
        let users = users::table.load::<User>(&self.pg_conn)?;
        Ok(users)
    }

    fn users_by(&self, by: &SearchBy) -> Result<Vec<Self::User>, Error> {
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

    fn users_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::User>, Error> {
        let users = users::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<User>(&self.pg_conn)?;

        Ok(users)
    }

    fn items(&self) -> Result<Vec<Self::Item>, Error> {
        let movies = movies::table.load::<Movie>(&self.pg_conn)?;
        Ok(movies)
    }

    fn items_by(&self, by: &SearchBy) -> Result<Vec<Self::Item>, Error> {
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

    fn items_offset_limit(&self, offset: usize, limit: usize) -> Result<Vec<Self::Item>, Error> {
        let items = movies::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Movie>(&self.pg_conn)?;

        Ok(items)
    }

    fn create_partial_users(
        &self,
        user_ids: &[eid!(Self::User)],
    ) -> Result<Vec<Self::User>, Error> {
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

    fn create_partial_items(
        &self,
        item_ids: &[eid!(Self::Item)],
    ) -> Result<Vec<Self::Item>, Error> {
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

    #[allow(clippy::type_complexity)]
    fn users_who_rated(
        &self,
        items: &[Self::Item],
    ) -> Result<maped_ratings!(Self::Item => Self::User), Error> {
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

    fn ratings_by(&self, user: &Self::User) -> Result<ratings!(Self::Item), Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .into_iter()
            .map(|rating| (rating.movie_id, rating.score))
            .collect();

        Ok(ratings)
    }

    #[allow(clippy::type_complexity)]
    fn maped_ratings(&self) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
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

    #[allow(clippy::type_complexity)]
    fn maped_ratings_by(
        &self,
        users: &[Self::User],
    ) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
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

    #[allow(clippy::type_complexity)]
    fn maped_ratings_except(
        &self,
        user: &Self::User,
    ) -> Result<maped_ratings!(Self::User => Self::Item), Error> {
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

    fn means_for(&self, users: &[Self::User]) -> Result<means!(Self::User), Error> {
        let means = Mean::belonging_to(users).load::<Mean>(&self.pg_conn)?;

        let means_by_user = means
            .into_iter()
            .map(|mean| (mean.user_id, mean.val))
            .collect();

        Ok(means_by_user)
    }

    fn score_range(&self) -> (f64, f64) {
        (1., 5.)
    }

    fn fields_for_users(&self) -> Vec<Field> {
        vec![Field::Required("name", Type::String)]
    }

    fn fields_for_items(&self) -> Vec<Field> {
        vec![Field::Required("name", Type::String)]
    }

    fn insert_user<'a>(&self, proto: HashMap<&'a str, Value>) -> Result<User, Error> {
        let user = NewUser {
            name: proto["name"].as_string()?,
        };

        Ok(insert_into(users::table)
            .values(&user)
            .get_result(&self.pg_conn)?)
    }

    fn insert_item<'a>(&self, proto: HashMap<&'a str, Value>) -> Result<Movie, Error> {
        let movie = NewMovie {
            name: proto["name"].as_string()?,
        };

        Ok(insert_into(movies::table)
            .values(&movie)
            .get_result(&self.pg_conn)?)
    }

    fn insert_rating(
        &self,
        user: &eid!(Self::User),
        item: &eid!(Self::Item),
        score: f64,
    ) -> Result<Self::Rating, Error> {
        todo!()
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
