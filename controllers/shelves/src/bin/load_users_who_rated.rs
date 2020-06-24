// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use indicatif::ProgressIterator;
use mongodb::bson::{doc, to_bson, Bson};
use mongodb::sync::Client;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let client = Client::with_uri_str(mongo_url)?;
    let db = client.database(mongo_db);
    let collection = db.collection("users_who_rated");

    let mut seen_items = HashSet::new();
    let chunk_size = 4000;

    loop {
        let file = File::open("data/goodreads_interactions.csv")?;
        let reader = BufReader::new(file);
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_reader(reader);
        let mut items_with_ratings = HashMap::new();
        for record in csv.records().progress() {
            if let Ok(record) = record {
                let user_id: i32 = record[0].parse()?;
                let movie_id: i32 = record[1].parse()?;
                let score: f64 = record[3].parse()?;

                if !seen_items.contains(&movie_id)
                    && (items_with_ratings.keys().len() < chunk_size
                        || items_with_ratings.contains_key(&movie_id))
                {
                    items_with_ratings
                        .entry(movie_id)
                        .or_insert_with(HashMap::new)
                        .insert(user_id.to_string(), Bson::Double(score));
                }
            }
        }

        println!(
            "Current items_with_ratings: {}",
            items_with_ratings.keys().len()
        );

        // Push the ratings vec when it's 10K length
        if !items_with_ratings.is_empty() {
            let mut docs = Vec::new();
            for (book_id, ratings) in items_with_ratings {
                let data = to_bson(&ratings)?;
                docs.push(doc! {"item_id": book_id, "scores": data});
                seen_items.insert(book_id);
            }

            collection.insert_many(docs, None)?;
        } else {
            break;
        }
    }
    Ok(())
}
