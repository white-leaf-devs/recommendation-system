use anyhow::Error;
use books::BooksController;
use controller::{Controller, Entity, SearchBy};
use movie_lens_small::MovieLensSmallController;
use recommend::distances::{post_adjusted_cosine, pre_adjusted_cosine};
use simple_movie::SimpleMovieController;
use std::env;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
#[error("Bad arguments, need at least 1")]
pub struct BadArgs;

fn calculate_sm<C, U, I>(controller: C) -> Result<(), Error>
where
    U: Entity,
    I: Entity,
    C: Controller<U, I>,
{
    let maped_ratings = controller.maped_ratings()?;
    let means = pre_adjusted_cosine(&maped_ratings);

    println!("{:#?}", means);

    let item_a = controller.items_by(&SearchBy::name("Alien"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Avatar"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("Star Wars"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Jaws"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("Pulp Fiction"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Braveheart"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("You Got Mail"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("The Matrix"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    Ok(())
}

fn calculate_mls<C, U, I>(controller: C) -> Result<(), Error>
where
    U: Entity,
    I: Entity,
    C: Controller<U, I>,
{
    let maped_ratings = controller.maped_ratings()?;
    let means = pre_adjusted_cosine(&maped_ratings);

    let item_a = controller.items_by(&SearchBy::name("Iron Will (1994)"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Friday (1995)"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("Room, The (2003)"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Dangerous Minds (1995)"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("Spider-Man (2002)"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Casino (1995)"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    let item_a = controller.items_by(&SearchBy::name("Multiplicity (1996)"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Forbidden Planet (1956)"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    Ok(())
}

fn calculate_b<C, U, I>(controller: C) -> Result<(), Error>
where
    U: Entity,
    I: Entity,
    C: Controller<U, I>,
{
    let maped_ratings = controller.maped_ratings()?;
    let means = pre_adjusted_cosine(&maped_ratings);

    let item_a = controller.items_by(&SearchBy::name("The yawning heights"))?[0].get_id();
    let item_b = controller.items_by(&SearchBy::name("Gangster"))?[0].get_id();
    println!(
        "{:?}",
        post_adjusted_cosine(&means, &maped_ratings, &item_a, &item_b)
    );

    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let db = args.get(1).ok_or(BadArgs)?;

    let now = Instant::now();
    match db.as_str() {
        "simple-movie" => {
            let controller = SimpleMovieController::new()?;
            calculate_sm(controller)?
        }

        "movie-lens-small" => {
            let controller = MovieLensSmallController::new()?;
            calculate_mls(controller)?
        }

        "books" => {
            let controller = BooksController::new()?;
            calculate_b(controller)?
        }

        not_supported => panic!("Not supported database: {}", not_supported),
    }

    println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());

    Ok(())
}
