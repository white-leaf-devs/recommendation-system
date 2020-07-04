// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use books::BooksController;
use config::Config;
use controller::Controller;
use indicatif::ProgressIterator;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::sync::Client;
use std::collections::{HashMap, HashSet};

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();
    let mut config = Config::default();

    let db = config.databases.get_mut("books").unwrap();
    db.psql_url = vars["DATABASE_URL"].clone();
    db.mongo_url = vars["MONGO_URL"].clone();
    db.mongo_db = vars["MONGO_DB"].clone();

    let client = Client::with_uri_str(&db.mongo_url)?;
    let users_who_rated = client.database(&db.mongo_db).collection("users_who_rated");
    let users_ratings = client.database(&db.mongo_db).collection("users_ratings");

    let controller = BooksController::from_config(&config, "books")?;
    let mut item_ids = HashSet::new();

    for items in controller.items_by_chunks(20000) {
        for item in items {
            item_ids.insert(item.id);
        }
    }

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_path("data/BX-Book-Ratings.csv")?;

    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let mut docs = HashMap::new();
    for record in records.into_iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let book_id = &record[1];
            let score: f64 = record[2].parse()?;

            if !item_ids.contains(book_id) {
                continue;
            }

            docs.entry(book_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(user_id.to_string(), Bson::Double(score));
        }
    }

    let docs: Vec<Document> = docs
        .into_iter()
        .map(|(k, v)| -> Result<_, Error> {
            let data = to_bson(&v)?;
            Ok(doc! { "item_id": k, "scores": data  })
        })
        .collect::<Result<_, Error>>()?;

    let chunk_size = docs.len() / 8;
    for chunk in docs.chunks(chunk_size) {
        let chunk = chunk.to_owned();
        users_who_rated.insert_many(chunk, None)?;
    }

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_path("data/BX-Book-Ratings.csv")?;

    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let mut docs = HashMap::new();
    for record in records.into_iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let book_id = &record[1];
            let score: f64 = record[2].parse()?;

            if !item_ids.contains(book_id) {
                continue;
            }

            docs.entry(user_id)
                .or_insert_with(HashMap::new)
                .insert(book_id.to_string(), Bson::Double(score));
        }
    }

    let docs: Vec<Document> = docs
        .into_iter()
        .map(|(k, v)| -> Result<_, Error> {
            let data = to_bson(&v)?;
            Ok(doc! { "user_id": k, "scores": data  })
        })
        .collect::<Result<_, Error>>()?;

    let chunk_size = docs.len() / 8;
    for chunk in docs.chunks(chunk_size) {
        let chunk = chunk.to_owned();
        users_ratings.insert_many(chunk, None)?;
    }

    Ok(())
}
