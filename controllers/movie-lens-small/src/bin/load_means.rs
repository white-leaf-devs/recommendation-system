// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
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

    let psql_url = &vars["DATABASE_URL"];
    let mongo_url = &vars["MONGO_URL"];
    let mongo_db = &vars["MONGO_DB"];
    let conn = establish_connection(psql_url)?;

    let controller = MovieLensSmallController::with_url(psql_url, mongo_url, mongo_db)?;

    let mut means = Vec::new();

    let users = controller.users()?;
    let maped_ratings = controller.maped_ratings_by(&users)?;

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
