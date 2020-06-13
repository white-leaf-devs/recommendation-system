use anyhow::Error;
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use indicatif::ProgressIterator;
use shelves::establish_connection;
use shelves::models::{books::NewBook, ratings::NewRating, users::NewUser};
use shelves::schema::{books, ratings, users};

fn insert_users(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/user_id_map.csv")?;

    let mut users = Vec::new();
    println!("Collecting records for users...");
    let records: Vec<_> = csv.records().collect();

    for record in records.iter().progress() {
        if let Ok(record) = record {
            let id: i32 = record[0].parse().map_err(|e| {
                println!("Failed for {}", &record[0]);
                e
            })?;

            users.push(NewUser { id });
        }
    }

    println!("Pushing users by chunks");
    for chunk in users.chunks(10_000).progress() {
        insert_into(users::table).values(chunk).execute(conn)?;
    }

    Ok(())
}

fn insert_books(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/book_id_map.csv")?;

    let mut books = Vec::new();
    println!("Collecting records for books...");
    let records: Vec<_> = csv.records().collect();

    for record in records.iter().progress() {
        if let Ok(record) = record {
            let id: i32 = record[0].parse().map_err(|e| {
                println!("Failed for {}", &record[0]);
                e
            })?;

            books.push(NewBook { id });
        }
    }

    println!("Pushing books by chunks");
    for chunk in books.chunks(10_000).progress() {
        insert_into(books::table).values(chunk).execute(conn)?;
    }

    Ok(())
}

fn insert_ratings(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path("data/goodreads_interactions.csv")?;

    let mut ratings = Vec::new();
    println!("Collecting records for ratings...");

    for record in csv.records().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let book_id: i32 = record[1].parse()?;
            let score: f64 = record[3].parse()?;

            ratings.push(NewRating {
                user_id,
                book_id,
                score,
            });
        }

        if !ratings.is_empty() && ratings.len() % 10_000 == 0 {
            insert_into(ratings::table).values(&ratings).execute(conn)?;
            ratings.clear();
        }
    }

    if !ratings.is_empty() {
        insert_into(ratings::table).values(&ratings).execute(conn)?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let url = "postgres://postgres:@localhost/shelves";
    let conn = establish_connection(url)?;

    insert_users(&conn)?;
    insert_books(&conn)?;
    insert_ratings(&conn)?;
    Ok(())
}
