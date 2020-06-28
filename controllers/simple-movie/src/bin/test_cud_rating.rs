// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use controller::{Controller, Value};
use simple_movie::SimpleMovieController;
use std::collections::HashMap;

fn test_insert() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let psql_url = &vars["DATABASE_URL"];
    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;

    let mut user = HashMap::new();
    user.insert("name", Value::String("Brus".into()));

    let user_result = controller.insert_user(user)?;
    println!("User insertion successful {:?}", user_result);

    let mut item = HashMap::new();
    item.insert(
        "name",
        Value::String("Hola soy el brus, la pelicula".into()),
    );

    let item_result = controller.insert_item(item)?;
    println!("Item insertion successful: {:?}", item_result);

    let user_id = 26;
    let item_id = 26;
    let score = 4.5;

    let rating_result = controller.insert_rating(&user_id, &item_id, score)?;
    println!("Rating insertion succesful: {:?}", rating_result);

    Ok(())
}

fn test_update() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let psql_url = &vars["DATABASE_URL"];
    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;

    let user_id = 26;
    let item_id = 26;
    let score = 3.0;

    let rating_result = controller.update_rating(&user_id, &item_id, score)?;
    println!("Rating update succesful: {:?}", rating_result);
    Ok(())
}

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let psql_url = &vars["DATABASE_URL"];
    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];

    let controller = SimpleMovieController::with_url(psql_url, mongo_url, mongo_db)?;

    let user_id = 26;
    let item_id = 26;

    let rating_result = controller.remove_rating(&user_id, &item_id);
    println!("Rating remove succesful: {:?}", rating_result);

    Ok(())
}
