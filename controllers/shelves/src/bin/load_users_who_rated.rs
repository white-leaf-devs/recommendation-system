// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use indicatif::ProgressIterator;
use mongodb::bson::{doc, to_bson, Bson};
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

    let file = File::open("data/ratings.csv")?;
    let reader = BufReader::new(file);
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);

    let mut current_item = None;
    let mut current_ratings = HashMap::new();

    for record in csv.records().progress() {
        if let Ok(record) = record {
            if record[0] == "user_id".to_string() {
                continue;
            }

            let user_id: i32 = record[0].parse()?;
            let book_id: i32 = record[1].parse()?;
            let score: f64 = record[3].parse()?;

            if let Some(current_item) = &mut current_item {
                if *current_item != book_id {
                    let data = to_bson(&current_ratings)?;
                    collection
                        .insert_one(doc! { "item_id": *current_item, "scores": data}, None)?;

                    *current_item = book_id;
                    current_ratings.clear();
                }
            } else {
                current_item = Some(book_id);
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

    Ok(())
}
