use anyhow::Error;
use controller::{Controller, Entity};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use movie_lens_small::establish_connection;
use movie_lens_small::models::{
    movies::{Movie, NewMovie},
    ratings::NewRating,
    users::{NewMean, NewUser, User},
};
use movie_lens_small::schema::{means, movies, ratings, users};
use movie_lens_small::MovieLensSmallController;
use std::collections::HashMap;

fn create_mean(conn: &PgConnection, user_id: i32, mean: f64) -> Result<(), Error> {
    let new_mean = NewMean { user_id, val: mean };

    insert_into(means::table).values(&new_mean).execute(conn)?;

    Ok(())
}

fn compute_mean(ratings: &HashMap<i32, f64>) -> f64 {
    if ratings.is_empty() {
        return 0.0;
    }

    let mut mean = 0.0;
    for (_, rating) in ratings {
        mean += rating;
    }
    mean / ratings.len() as f64
}

fn main() -> Result<(), Error> {
    let url = "postgres://postgres:@localhost/movie-lens-small";
    let conn = establish_connection(url)?;

    let controller = MovieLensSmallController::new()?;

    let users_iterator = controller.users_by_chunks(10000);
    for user_chunk in users_iterator {
        let maped_ratings = controller.maped_ratings_by(&user_chunk)?;
        for user in user_chunk {
            let user_id = user.get_id();
            if maped_ratings.contains_key(&user_id) {
                let mean = compute_mean(&maped_ratings[&user_id]);
                create_mean(&conn, user_id, mean)?;
            } else {
                create_mean(&conn, user_id, 0.0)?;
            }
        }
    }

    Ok(())
}
