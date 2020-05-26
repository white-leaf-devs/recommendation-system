use anyhow::Error;
use controller::Controller;
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use indicatif::ProgressIterator;
use movie_lens_small::establish_connection;
use movie_lens_small::models::{movies::NewMovie, ratings::NewRating, users::NewUser};
use movie_lens_small::schema::{movies, ratings, users};
use movie_lens_small::MovieLensSmallController;

fn insert_users(conn: &PgConnection) -> Result<(), Error> {
    let mut users = Vec::new();
    println!("Collecting records for users...");

    for id in 1..=610 {
        users.push(NewUser { id });
    }

    println!("Pushing into the database");
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

fn insert_ratings(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/ratings.csv")?;

    let mut ratings = Vec::new();
    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let controller = MovieLensSmallController::new()?;
    for record in records.iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let movie_id: i32 = record[1].parse()?;
            let score: f64 = record[2].parse()?;

            let book_res = controller.item_by_id(&movie_id.into());
            if book_res.is_err() {
                continue;
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
    let url = "postgres://postgres:@localhost/movie-lens-small";
    let conn = establish_connection(url)?;

    insert_users(&conn)?;
    insert_movies(&conn)?;
    insert_ratings(&conn)?;
    Ok(())
}
