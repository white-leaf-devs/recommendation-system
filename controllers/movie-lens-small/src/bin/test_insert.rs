// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use controller::{Controller, Value};
use movie_lens_small::MovieLensSmallController;
use std::collections::HashMap;

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let psql_url = &vars["DATABASE_URL"];
    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let controller = MovieLensSmallController::with_url(psql_url, mongo_url, mongo_db)?;

    let user_result = controller.insert_user(HashMap::new())?;
    println!("User insertion successful {:?}", user_result);

    let mut item = HashMap::new();
    item.insert(
        "title",
        Value::String("Hola soy el brus, la pelicula".to_string()),
    );
    item.insert(
        "genres",
        Value::String("Acción, Drama, Romance".to_string()),
    );

    let item_result = controller.insert_item(item)?;
    println!("Item insertion successful: {:?}", item_result);

    Ok(())
}
