#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::{movies::Movie, ratings::Rating, users::User};
use crate::schema::{movies, ratings, users};
use anyhow::Error;
use controller::{Controller, Id, MapedRatings, Ratings};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::Hash,
};

pub fn establish_connection(url: &str) -> Result<PgConnection, Error> {
    Ok(PgConnection::establish(&url)?)
}

pub struct MovieLensSmallController {
    pg_conn: PgConnection,
    hasher_builder: RandomState,
}

impl MovieLensSmallController {
    pub fn new() -> Result<Self, Error> {
        Self::with_url("postgres://postgres:@localhost/movie-lens-small")
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        let pg_conn = establish_connection(url)?;
        Ok(Self {
            pg_conn,
            hasher_builder: Default::default(),
        })
    }
}

impl Controller<User, Movie> for MovieLensSmallController {
    fn make_hash<K: Hash>(&self, k: K) -> u64 {
        controller::make_hash(&self.hasher_builder, k)
    }

    fn user_by_id(&self, id: &Id) -> Result<User, Error> {
        let id: i32 = id.parse()?;

        let user = users::table
            .filter(users::id.eq(id))
            .first::<User>(&self.pg_conn)?;

        Ok(user)
    }

    fn item_by_id(&self, id: &Id) -> Result<Movie, Error> {
        let id: i32 = id.parse()?;

        let movie = movies::table
            .filter(movies::id.eq(id))
            .first::<Movie>(&self.pg_conn)?;

        Ok(movie)
    }

    fn item_by_name(&self, name: &str) -> Result<Vec<Movie>, Error> {
        let movies: Vec<Movie> = movies::table
            .filter(movies::title.eq(name))
            .load(&self.pg_conn)?;

        Ok(movies)
    }

    fn ratings_by_user(&self, user: &User) -> Result<Ratings, Error> {
        let ratings: HashMap<_, _> = Rating::belonging_to(user)
            .load::<Rating>(&self.pg_conn)?
            .iter()
            .map(|rating| {
                let movie_id = self.make_hash(rating.movie_id);
                (movie_id, rating.score)
            })
            .collect();

        Ok(ratings)
    }

    fn ratings_except_for(&self, user: &User) -> Result<MapedRatings, Error> {
        let ratings: Vec<Rating> = ratings::table
            .filter(ratings::user_id.is_distinct_from(user.id))
            .load(&self.pg_conn)?;

        let mut maped_ratings = HashMap::new();
        for rating in ratings {
            let movie_id = self.make_hash(rating.movie_id);

            maped_ratings
                .entry(rating.user_id.into())
                .or_insert_with(HashMap::new)
                .insert(movie_id, rating.score);
        }

        Ok(maped_ratings)
    }
}
