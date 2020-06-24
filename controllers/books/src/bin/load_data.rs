// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use books::establish_connection;
use books::models::{books::NewBook, ratings::NewRating, users::NewUser};
use books::schema::{books as books_sc, ratings, users};
use books::BooksController;
use controller::{Controller, SearchBy};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use indicatif::ProgressIterator;
use std::collections::HashMap;

fn insert_users(conn: &PgConnection) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_path("data/BX-Users.csv")?;

    let mut users = Vec::new();
    println!("Collecting records for users...");
    let records: Vec<_> = csv.records().collect();

    for record in records.iter().progress() {
        if let Ok(record) = record {
            let id: i32 = record[0].parse()?;
            let location = &record[1];
            let age: Option<i16> = if &record[2] == "\\N" {
                None
            } else {
                Some(record[2].parse()?)
            };

            users.push(NewUser { id, location, age });
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
        .has_headers(false)
        .delimiter(b';')
        .from_path("data/BX-Books.csv")?;

    let mut books = Vec::new();
    println!("Collecting records for books...");
    let records: Vec<_> = csv.records().collect();

    for record in records.iter().progress() {
        if let Ok(record) = record {
            let id = &record[0];
            let title = &record[1];
            let author = &record[2];
            let year: i16 = record[3].parse()?;
            let publisher = &record[4];

            books.push(NewBook {
                id,
                title,
                author,
                year,
                publisher,
            });
        }
    }

    println!("Pushing books by chunks");
    for chunk in books.chunks(10_000).progress() {
        insert_into(books_sc::table).values(chunk).execute(conn)?;
    }

    Ok(())
}

fn insert_ratings(conn: &PgConnection, url: &str) -> Result<(), Error> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_path("data/BX-Book-Ratings.csv")?;

    let mut ratings = Vec::new();
    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let controller = BooksController::with_url(url, "", "")?;
    for record in records.iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let book_id = &record[1];
            let score: f64 = record[2].parse()?;

            match controller.items_by(&SearchBy::id(&book_id)) {
                Ok(books) if books.is_empty() => continue,
                Err(_) => continue,
                Ok(_) => {}
            }

            ratings.push(NewRating {
                score,
                user_id,
                book_id,
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

    let url = &vars["DATABASE_URL"];
    let conn = establish_connection(url)?;

    insert_users(&conn)?;
    insert_books(&conn)?;
    insert_ratings(&conn, url)?;
    Ok(())
}
