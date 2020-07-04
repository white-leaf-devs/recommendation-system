// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use config::Config;
use controller::Controller;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::sync::Client;
use simple_movie::SimpleMovieController;
use std::collections::HashMap;

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();
    let mut config = Config::default();

    let db = config.databases.get_mut("simple-movie").unwrap();
    db.use_postgres = true;
    db.psql_url = vars["DATABASE_URL"].clone();
    db.mongo_url = vars["MONGO_URL"].clone();
    db.mongo_db = vars["MONGO_DB"].clone();

    let client = Client::with_uri_str(&db.mongo_url)?;
    let users_who_rated = client.database(&db.mongo_db).collection("users_who_rated");
    let users_ratings = client.database(&db.mongo_db).collection("users_ratings");

    let controller = SimpleMovieController::from_config(&config, "simple-movie")?;

    let mut docs = HashMap::new();
    for (user_id, ratings) in controller.all_users_ratings()? {
        for (item_id, score) in ratings {
            docs.entry(item_id)
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

    users_who_rated.insert_many(docs, None)?;

    // Inserting the inverse of above (It was item_id=>user_id now is user_id=>item_id)

    let mut docs = HashMap::new();
    for (user_id, ratings) in controller.all_users_ratings()? {
        for (item_id, score) in ratings {
            docs.entry(user_id)
                .or_insert_with(HashMap::new)
                .insert(item_id.to_string(), Bson::Double(score));
        }
    }

    let docs: Vec<Document> = docs
        .into_iter()
        .map(|(k, v)| -> Result<_, Error> {
            let data = to_bson(&v)?;
            Ok(doc! { "user_id": k, "scores": data  })
        })
        .collect::<Result<_, Error>>()?;

    users_ratings.insert_many(docs, None)?;

    Ok(())
}
