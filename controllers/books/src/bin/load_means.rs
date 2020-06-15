use anyhow::Error;
use books::establish_connection;
use books::models::{
    books::{Book, NewBook},
    ratings::NewRating,
    users::{NewMean, NewUser, User},
};
use books::schema::{books as books_sc, means, ratings, users};
use books::BooksController;
use controller::{Controller, Entity};
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use std::collections::HashMap;

fn insert_means(conn: &PgConnection, new_means: &Vec<NewMean>) -> Result<(), Error> {
    insert_into(means::table).values(new_means).execute(conn)?;

    Ok(())
}

fn compute_mean(ratings: &HashMap<String, f64>) -> f64 {
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
    let url = "postgres://postgres:@localhost/books";
    let conn = establish_connection(url)?;

    let controller = BooksController::new()?;

    let users_iterator = controller.users_by_chunks(10000);
    for user_chunk in users_iterator {
        println!("Inserting new chunk");
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
    }

    Ok(())
}