use anyhow::Error;
use controller::Controller;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::sync::Client;
use simple_movie::SimpleMovieController;
use std::collections::HashMap;

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];
    let psql_url = &vars["DATABASE_URL"];

    let client = Client::with_uri_str(mongo_url)?;
    let db = client.database(mongo_db);
    let collection = db.collection("users_who_rated");

    let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;

    let mut docs = HashMap::new();
    for (user_id, ratings) in controller.maped_ratings()? {
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

    collection.insert_many(docs, None)?;

    Ok(())
}
