// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use config::Config;
use controller::{Controller, SearchBy};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use indicatif::ProgressIterator;
use movie_lens_small::establish_connection;
use movie_lens_small::models::{movies::NewMovie, ratings::NewRating, users::NewUser};
use movie_lens_small::schema::{movies, ratings, users};
use movie_lens_small::MovieLensSmallController;
use std::collections::HashMap;

fn insert_users(conn: &PgConnection) -> Result<(), Error> {
    println!("Collecting records for users...");

    let users: Vec<_> = (1..=610).map(|id| NewUser { id }).collect();
    insert_into(users::table).values(&users).execute(conn)?;

    Ok(())
}

fn insert_movies(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/movies.csv")?;

    let mut movies = Vec::new();
    println!("Collecting records for movies...");
    let records: Vec<_> = csv.records().collect();

    for record in records.iter().progress() {
        if let Ok(record) = record {
            let id: i32 = record[0].parse().map_err(|e| {
                println!("Failed for {}", &record[0]);
                e
            })?;
            let title = &record[1];
            let genres = &record[2];

            movies.push(NewMovie { id, title, genres });
        }
    }

    println!("Pushing into the database");
    insert_into(movies::table).values(&movies).execute(conn)?;

    Ok(())
}

fn insert_ratings(conn: &PgConnection, config: &Config) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/ratings.csv")?;

    let mut ratings = Vec::new();
    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let controller = MovieLensSmallController::from_config(config, "movie-lens-small")?;
    for record in records.iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let movie_id: i32 = record[1].parse()?;
            let score: f64 = record[2].parse()?;

            match controller.items_by(&SearchBy::id(&movie_id.to_string())) {
                Ok(movies) if movies.is_empty() => continue,
                Err(_) => continue,
                Ok(_) => {}
            }

            ratings.push(NewRating {
                score,
                user_id,
                movie_id,
            });
        }
    }

    println!("Pushing ratings by chunks");
    for chunk in ratings.chunks(10_000).progress() {
        insert_into(ratings::table).values(chunk).execute(conn)?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();
    let mut config = Config::default();

    let db = config.databases.get_mut("movie-lens-small").unwrap();
    db.psql_url = vars["DATABASE_URL"].clone();
    db.mongo_url = vars["MONGO_URL"].clone();
    db.mongo_db = vars["MONGO_DB"].clone();

    let conn = establish_connection(&db.psql_url)?;

    insert_users(&conn)?;
    insert_movies(&conn)?;
    insert_ratings(&conn, &config)?;
    Ok(())
}
