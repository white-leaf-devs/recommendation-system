use anyhow::Error;
use diesel::pg::PgConnection;
use diesel::{insert_into, prelude::*};
use simple_movie::establish_connection;
use simple_movie::models::{Movie, NewMovie, NewRating, NewUser, User};
use simple_movie::schema::{movies, ratings, users};

fn create_movie(conn: &PgConnection, name: &str) -> Result<Movie, Error> {
    let new_movie = NewMovie { name };

    let movie = insert_into(movies::table)
        .values(&new_movie)
        .get_result(conn)?;

    Ok(movie)
}

fn create_user(conn: &PgConnection, name: &str) -> Result<User, Error> {
    let new_user = NewUser { name };

    let user = insert_into(users::table)
        .values(&new_user)
        .get_result(conn)?;

    Ok(user)
}

fn create_rating(
    conn: &PgConnection,
    score: f64,
    user_id: i32,
    movie_id: i32,
) -> Result<(), Error> {
    let new_rating = NewRating {
        score,
        user_id,
        movie_id,
    };

    insert_into(ratings::table)
        .values(&new_rating)
        .execute(conn)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let url = "postgres://postgres:@localhost/simple-movie";
    let conn = establish_connection(url);

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("data/movies.csv")?;

    let mut raw_table = Vec::new();

    for record in csv.records() {
        if let Ok(record) = record {
            let mut row = Vec::new();
            for val in record.iter() {
                row.push(val.to_string());
            }
            raw_table.push(row);
        }
    }

    let users = raw_table[0].iter().skip(1);
    let mut users_id = Vec::new();
    for user in users {
        let user = create_user(&conn, user)?;
        users_id.push(user.id);
    }

    let ratings = raw_table.iter().skip(1);
    for rating in ratings {
        let movie = &rating[0];
        let movie = create_movie(&conn, movie)?;

        for (i, value) in rating.iter().skip(1).enumerate() {
            if value.is_empty() {
                continue;
            }

            let value: f64 = value.parse()?;
            create_rating(&conn, value, users_id[i], movie.id)?;
        }
    }

    Ok(())
}
