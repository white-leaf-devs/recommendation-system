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
use controller::{error::ErrorKind, Controller, MapedRatings, Ratings, SearchBy};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::collections::HashMap;

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct MovieLensSmallController {
    pg_conn: PgConnection,
}

impl MovieLensSmallController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url("postgres://postgres:@localhost/movie-lens-small")
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(url)?;
        Ok(Self { pg_conn })
    }
}

impl Controller<User, i32, Movie, i32> for MovieLensSmallController {
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

    fn ratings_by(&self, user: &User) -> Result<Ratings<i32>, Error> {
        let ratings = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
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

    fn users_who_rated(&self, items: &[Movie]) -> Result<MapedRatings<i32, i32>, Error> {
        let ratings = Rating::belonging_to(items).load::<Rating>(&self.pg_conn)?;
        let mut items_users = HashMap::new();

        for rating in ratings {
            items_users
                .entry(rating.movie_id)
                .or_insert_with(HashMap::new)
                .insert(rating.user_id, rating.score);
        }
        Ok(items_users)
    }

    fn create_partial_users(&self, user_ids: &[i32]) -> Result<Vec<User>, Error> {
        user_ids
            .iter()
            .map(|id| -> Result<User, Error> { Ok(User { id: *id }) })
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
        (0.5, 5.)
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
