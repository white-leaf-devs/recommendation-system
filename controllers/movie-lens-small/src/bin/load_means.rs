// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use config::Config;
use controller::Controller;
use diesel::{insert_into, prelude::*};
use movie_lens_small::establish_connection;
use movie_lens_small::models::users::NewMean;
use movie_lens_small::schema::means;
use movie_lens_small::MovieLensSmallController;
use std::collections::HashMap;

fn compute_mean(ratings: &HashMap<i32, f64>) -> Option<f64> {
    if ratings.is_empty() {
        return None;
    }

    let mut mean = 0.0;
    for rating in ratings.values() {
        mean += rating;
    }

    Some(mean / ratings.len() as f64)
}

fn main() -> Result<(), Error> {
    let vars: HashMap<String, String> = dotenv::vars().collect();
    let mut config = Config::default();

    let db = config.databases.get_mut("movie-lens-small").unwrap();
    db.psql_url = vars["DATABASE_URL"].clone();
    db.mongo_url = vars["MONGO_URL"].clone();
    db.mongo_db = vars["MONGO_DB"].clone();

    let conn = establish_connection(&db.psql_url)?;
    let controller = MovieLensSmallController::from_config(&config, "movie-lens-small")?;

    let mut means = Vec::new();

    let users = controller.users()?;
    let maped_ratings = controller.users_ratings(&users)?;

    for (user_id, ratings) in maped_ratings {
        let mean = compute_mean(&ratings);

        if let Some(mean) = mean {
            let len = ratings.len();

            let mean = NewMean {
                user_id,
                val: mean,
                score_number: len as i32,
            };

            means.push(mean);
        }
    }

    insert_into(means::table).values(&means).execute(&conn)?;

    Ok(())
}
