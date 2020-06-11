use anyhow::Error;
use controller::{Controller, Entity};
use movie_lens_small::MovieLensSmallController;
use recommend::distances::{post_adjusted_cosine, pre_adjusted_cosine};
use simple_movie::SimpleMovieController;
use std::collections::HashMap;
use std::env;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
#[error("Bad arguments, need at least 1")]
pub struct BadArgs;

fn calc_matrix<C, U, I>(controller: C) -> Result<HashMap<String, HashMap<String, f64>>, Error>
where
    U: Entity,
    I: Entity,
    C: Controller<U, I>,
{
    let items = controller.items()?;
    let maped_ratings = controller.maped_ratings()?;

    let mut matrix = HashMap::new();

    println!("Calculating for {} items", items.len());
    let means = pre_adjusted_cosine(&maped_ratings);

    for (i, item_a) in items.iter().enumerate() {
        println!("Calculating row for item with id('{}')", item_a.get_id());
        for item_b in items.iter().skip(i + 1) {
            let val =
                post_adjusted_cosine(&means, &maped_ratings, &item_a.get_id(), &item_b.get_id());

            if let Some(val) = val {
                matrix
                    .entry(item_a.get_id())
                    .or_insert_with(HashMap::new)
                    .insert(item_b.get_id(), val);
            }
        }
    }

    Ok(matrix)
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let db = args.get(1).ok_or(BadArgs)?;

    let now = Instant::now();
    let matrix = match db.as_str() {
        "simple-movie" => {
            let controller = SimpleMovieController::new()?;
            calc_matrix(controller)?
        }

        "movie-lens-small" => {
            let controller = MovieLensSmallController::new()?;
            calc_matrix(controller)?
        }

        not_supported => panic!("Not supported database: {}", not_supported),
    };

    println!("{:#?}", matrix);
    println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());

    Ok(())
}
