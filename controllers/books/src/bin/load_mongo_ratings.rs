// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use books::BooksController;
use controller::Controller;
use indicatif::ProgressIterator;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::sync::Client;
use std::collections::{HashMap, HashSet};

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];
    let psql_url = &vars["DATABASE_URL"];

    let client = Client::with_uri_str(mongo_url)?;
    let db = client.database(mongo_db);
    let collection = db.collection("users_who_rated");

    let controller = BooksController::with_url(psql_url, mongo_url, mongo_db)?;
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
        collection.insert_many(chunk, None)?;
    }

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_path("data/BX-Book-Ratings.csv")?;

    println!("Collecting records for ratings...");
    let records: Vec<_> = csv.records().collect();

    let collection = db.collection("user_ratings");
    let mut docs = HashMap::new();
    for record in records.into_iter().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let book_id = &record[1];
            let score: f64 = record[2].parse()?;

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
        collection.insert_many(chunk, None)?;
    }

    Ok(())
}
