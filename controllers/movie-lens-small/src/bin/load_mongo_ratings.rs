// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use indicatif::ProgressIterator;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::sync::Client;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let client = Client::with_uri_str(mongo_url)?;
    let db = client.database(mongo_db);
    let collection = db.collection("users_who_rated");

    let file = File::open("data/ratings-by-items.csv")?;
    let reader = BufReader::new(file);
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);

    let mut current_item = None;
    let mut current_ratings = HashMap::new();

    for record in csv.records().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let movie_id: i32 = record[1].parse()?;
            let score: f64 = record[2].parse()?;

            if let Some(current_item) = &mut current_item {
                if *current_item != movie_id {
                    let data = to_bson(&current_ratings)?;
                    collection
                        .insert_one(doc! { "item_id": *current_item, "scores": data }, None)?;

                    *current_item = movie_id;
                    current_ratings.clear();
                }
            } else {
                current_item = Some(movie_id);
            }

            current_ratings.insert(user_id.to_string(), Bson::Double(score));
        }
    }

    if let Some(current_item) = current_item {
        if !current_ratings.is_empty() {
            let data = to_bson(&current_ratings)?;
            collection.insert_one(doc! { "item_id": current_item, "scores": data}, None)?;
        }
    }

    let collection = db.collection("user_ratings");

    let file = File::open("data/ratings.csv")?;
    let reader = BufReader::new(file);
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);

    let mut docs = HashMap::new();
    for record in csv.records().progress() {
        if let Ok(record) = record {
            let user_id: i32 = record[0].parse()?;
            let movie_id: i32 = record[1].parse()?;
            let score: f64 = record[2].parse()?;

            docs.entry(user_id)
                .or_insert_with(HashMap::new)
                .insert(movie_id.to_string(), Bson::Double(score));
        }
    }

    let docs: Vec<Document> = docs
        .into_iter()
        .map(|(k, v)| -> Result<_, Error> {
            let data = to_bson(&v)?;
            Ok(doc! { "user_id": k, "scores": data  })
        })
        .collect::<Result<_, Error>>()?;

    for chunk in docs.chunks(10_000).progress() {
        let chunk = chunk.to_owned();
        collection.insert_many(chunk, None)?;
    }
    Ok(())
}
