// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use controller::{Controller, Entity};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use shelves::establish_connection;
use shelves::models::users::NewMean;
use shelves::schema::means;
use shelves::ShelvesController;
use std::collections::HashMap;
use std::time::Instant;

fn insert_means(conn: &PgConnection, new_means: &[NewMean]) -> Result<(), Error> {
    insert_into(means::table).values(new_means).execute(conn)?;

    Ok(())
}

fn compute_mean(ratings: &HashMap<i32, f64>) -> f64 {
    if ratings.is_empty() {
        return 0.0;
    }

    let mut mean = 0.0;
    for rating in ratings.values() {
        mean += rating;
    }
    mean / ratings.len() as f64
}

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();

    let url = &vars["DATABASE_URL"];
    let conn = establish_connection(url)?;

    let controller = ShelvesController::with_url(url, "", "")?;

    let users_iterator = controller.users_by_chunks(10000);
    for user_chunk in users_iterator {
        println!("Inserting new chunk");
        let now = Instant::now();
        let mut mean_chunk = Vec::new();
        let maped_ratings = controller.maped_ratings_by(&user_chunk)?;
        for user in user_chunk {
            let user_id = user.get_id();
            if maped_ratings.contains_key(&user_id) {
                let mean = compute_mean(&maped_ratings[&user_id]);
                mean_chunk.push(NewMean { user_id, val: mean });
            } else {
                mean_chunk.push(NewMean { user_id, val: 0.0 });
            }
        }
        insert_means(&conn, &mean_chunk)?;
        println!("Elapsed per iteration: {}", now.elapsed().as_secs_f64());
    }

    Ok(())
}
