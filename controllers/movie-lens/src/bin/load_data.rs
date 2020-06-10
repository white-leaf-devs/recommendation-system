use anyhow::Error;
use controller::{Controller, SearchBy};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use indicatif::ProgressIterator;
use movie_lens::establish_connection;
use movie_lens::models::{movies::NewMovie, ratings::NewRating, users::NewUser};
use movie_lens::schema::{movies, ratings, users};
use movie_lens::MovieLensController;
use std::fs::File;
use std::io::BufReader;

fn insert_users(conn: &PgConnection) -> Result<(), Error> {
    let mut users = Vec::new();
    println!("Collecting records for users...");

    for id in 1..=283_228 {
        users.push(NewUser { id });
    }

    println!("Pushing users by chunks");
    for chunk in users.chunks(10_000).progress() {
        insert_into(users::table).values(chunk).execute(conn)?;
    }

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

    println!("Pushing ratings by chunks");
    for chunk in movies.chunks(10_000).progress() {
        insert_into(movies::table).values(chunk).execute(conn)?;
    }

    Ok(())
}

fn insert_ratings(conn: &PgConnection) -> Result<(), Error> {
    let file = File::open("data/ratings.csv")?;
    let reader = BufReader::new(file);

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);

    let mut ratings = Vec::new();

    println!("Collecting records for ratings...");
    let controller = MovieLensController::new()?;
    for record in csv.records().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let movie_id: i32 = record[1].parse()?;
            let score: f64 = record[2].parse()?;

            ratings.push(NewRating {
                score,
                user_id,
                movie_id,
            });
        }

        // Push the ratings vec when it's 10K length
        if !ratings.is_empty() && ratings.len() % 10_000 == 0 {
            insert_into(ratings::table).values(&ratings).execute(conn)?;

            // Clear ratings for the following iterations
            ratings.clear();
        }
    }

    if !ratings.is_empty() {
        insert_into(ratings::table).values(&ratings).execute(conn)?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let url = "postgres://postgres:@localhost/movie-lens";
    let conn = establish_connection(url)?;

    insert_users(&conn)?;
    insert_movies(&conn)?;
    insert_ratings(&conn)?;
    Ok(())
}
