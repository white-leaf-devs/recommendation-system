pub mod parser;

use anyhow::Error;
use books::BooksController;
use config::Config;
use controller::{Controller, Entity, ToTable};
use engine::{similarity_matrix::SimilarityMatrix, Engine};
use movie_lens::MovieLensController;
use movie_lens_small::MovieLensSmallController;
use parser::{Database, Statement};
use rustyline::Editor;
use shelves::ShelvesController;
use simple_movie::SimpleMovieController;
use simplelog::{
    CombinedLogger, Config as LogConfig, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{fmt::Display, fs::File, hash::Hash, time::Instant};

macro_rules! prompt {
    ($ed:ident) => {{
        prompt!($ed, "")
    }};

    ($ed:ident, $db:expr) => {{
        use rustyline::error::ReadlineError;

        let msg = if $db.is_empty() {
            format!("{}", PROMPT)
        } else {
            format!("({}) {}", $db, PROMPT)
        };

        match $ed.readline(&msg) {
            Ok(line) => {
                $ed.add_history_entry(line.as_str());
                Ok(line)
            }

            Err(ReadlineError::Interrupted) => {
                continue;
            }

            Err(ReadlineError::Eof) => {
                if $db.is_empty() {
                    println!("Exiting...Good bye!");
                } else {
                    println!("Disconnecting from {}", $db);
                }

                break;
            }

            Err(e) => Err(e),
        }
    }};
}

fn sim_matrix_prompt<C, User, UserId, Item, ItemId>(
    controller: &C,
    name: &str,
    ver_chunk_size: usize,
    hor_chunk_size: usize,
    rl: &mut Editor<()>,
) -> Result<(), Error>
where
    C: Controller<User, UserId, Item, ItemId>,
    User: Entity<Id = UserId> + ToTable,
    Item: Entity<Id = ItemId> + ToTable,
    UserId: Hash + Eq + Display + Clone + Default,
    ItemId: Hash + Eq + Display + Clone,
{
    let config = Config::load("config.toml")?;
    let mut sim_matrix = SimilarityMatrix::new(controller, &config, ver_chunk_size, hor_chunk_size);
    let mut curr_i = 0;
    let mut curr_j = 0;

    let now = Instant::now();
    let mut chunk = match sim_matrix.get_chunk(curr_i, curr_j) {
        Ok(chunk) => chunk,
        Err(e) => {
            log::error!("{}", e);
            return Ok(());
        }
    };
    println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());

    loop {
        let formatted = format!("{}:sim_matrix({}, {})", name, curr_i, curr_j);
        let opt: String = prompt!(rl, formatted)?;

        match opt.trim() {
            "e" | "exit" => {
                println!("Exiting the matrix");
                break;
            }

            "v" | "version" => {
                println!("version: {}", VERSION);
            }

            line => match parser::parse_line(line) {
                Some(stmt) => match stmt {
                    Statement::SimMatrixGet(searchby_a, searchby_b) => {
                        let item_id_a = match controller.items_by(&searchby_a) {
                            Ok(items) => items[0].get_id(),
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item_id_b = match controller.items_by(&searchby_b) {
                            Ok(items) => items[0].get_id(),
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let val = chunk.get(&item_id_a).and_then(|row| row.get(&item_id_b));

                        if let Some(val) = val {
                            println!("Value for ({}, {}) is {}", item_id_a, item_id_b, val);
                        } else {
                            log::error!("No value found for ({}, {})", item_id_a, item_id_b);
                        }
                    }

                    Statement::SimMatrixMoveTo(i, j) => {
                        let old_i = curr_i;
                        let old_j = curr_j;

                        curr_i = i;
                        curr_j = j;

                        let now = Instant::now();
                        chunk = match sim_matrix.get_chunk(curr_i, curr_j) {
                            Ok(chunk) => chunk,
                            Err(e) => {
                                log::error!("{}", e);
                                curr_i = old_i;
                                curr_j = old_j;
                                chunk
                            }
                        };
                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    _ => {
                        log::error!("Invalid statement in this context.");
                        log::error!("Exit the matrix first!");
                    }
                },

                None => log::error!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}

fn database_connected_prompt<C, User, UserId, Item, ItemId>(
    controller: C,
    name: &str,
    rl: &mut Editor<()>,
) -> Result<(), Error>
where
    C: Controller<User, UserId, Item, ItemId>,
    User: Entity<Id = UserId> + ToTable + Clone,
    Item: Entity<Id = ItemId> + ToTable + Clone,
    UserId: Hash + Eq + Display + Clone + Default,
    ItemId: Hash + Eq + Display + Clone,
{
    let config = Config::load("config.toml")?;
    let engine = Engine::with_controller(&controller, &config);

    loop {
        let opt: String = prompt!(rl, name)?;

        match opt.trim() {
            "d" | "disconnect" => {
                println!("Disconnecting from database {}", name);
                break;
            }

            "v" | "version" => {
                println!("version: {}", VERSION);
            }

            line => match parser::parse_line(line) {
                Some(stmt) => match stmt {
                    Statement::Connect(_) => {
                        log::error!("Invalid statement in this context.");
                        log::error!("Disconnect from current database first!");
                    }

                    Statement::SimMatrixGet(_, _) | Statement::SimMatrixMoveTo(_, _) => {
                        log::error!("Invalid statement in this context.");
                        log::error!("Enter the matrix first!");
                    }

                    Statement::QueryUser(searchby) => match controller.users_by(&searchby) {
                        Ok(users) => {
                            for user in users {
                                println!("{}", user.to_table());
                            }
                        }
                        Err(e) => log::error!("{}", e),
                    },

                    Statement::QueryItem(searchby) => match controller.items_by(&searchby) {
                        Ok(items) => {
                            for item in items {
                                println!("{}", item.to_table());
                            }
                        }
                        Err(e) => log::error!("{}", e),
                    },

                    Statement::QueryRatings(searchby) => match controller.users_by(&searchby) {
                        Ok(users) => {
                            for user in users {
                                if let Ok(ratings) = controller.ratings_by(&user) {
                                    if !ratings.is_empty() {
                                        println!("{}", ratings.to_table());
                                    } else {
                                        log::error!(
                                            "No ratings found for user with id({})",
                                            user.get_id()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => log::error!("{}", e),
                    },

                    Statement::ItemDistance(searchby_a, searchby_b, method) => {
                        let item_a = match controller
                            .items_by(&searchby_a)
                            .map(|mut items| items.drain(..1).next().unwrap())
                        {
                            Ok(item) => item,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item_b = match controller
                            .items_by(&searchby_b)
                            .map(|mut items| items.drain(..1).next().unwrap())
                        {
                            Ok(item) => item,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let now = Instant::now();
                        let dist = engine.item_distance(item_a, item_b, method);
                        match dist {
                            Ok(dist) => println!("Distance is {}", dist),
                            Err(e) => {
                                log::error!("Distance couldn't be calculated");
                                log::error!("Reason: {}", e);
                            }
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::UserDistance(searchby_a, searchby_b, method) => {
                        let user_a = match controller
                            .users_by(&searchby_a)
                            .map(|mut users| users.drain(..1).next().unwrap())
                        {
                            Ok(user) => user,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let user_b = match controller
                            .users_by(&searchby_b)
                            .map(|mut users| users.drain(..1).next().unwrap())
                        {
                            Ok(user) => user,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let now = Instant::now();
                        let dist = engine.user_distance(user_a, user_b, method);
                        match dist {
                            Ok(dist) => println!("Distance is {}", dist),
                            Err(e) => {
                                log::error!("Distance couldn't be calculated");
                                log::error!("Reason: {}", e);
                            }
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::UserKnn(k, searchby, method, chunks_opt) => {
                        let user = match controller
                            .users_by(&searchby)
                            .map(|mut users| users.drain(..1).next().unwrap())
                        {
                            Ok(user) => user,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let now = Instant::now();
                        let knn = engine.user_knn(k, user, method, chunks_opt);

                        let elapsed = now.elapsed().as_secs_f64();

                        match knn {
                            Ok(knn) => {
                                for (nn_id, dist) in knn {
                                    println!("Distance with user with id({}) is {}", nn_id, dist);
                                }
                            }

                            Err(e) => {
                                log::error!("Failed to find the {} nearest neighbors", k);
                                log::error!("Reason: {}", e);
                            }
                        }

                        println!("Operation took {:.4} seconds", elapsed);
                    }

                    Statement::UserBasedPredict(
                        k,
                        searchby_user,
                        searchby_item,
                        method,
                        chunks_opt,
                    ) => {
                        let user = match controller
                            .users_by(&searchby_user)
                            .map(|mut users| users.drain(..1).next().unwrap())
                        {
                            Ok(user) => user,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item = match controller
                            .items_by(&searchby_item)
                            .map(|mut items| items.drain(..1).next().unwrap())
                        {
                            Ok(item) => item,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item_id = item.get_id();

                        let now = Instant::now();
                        let prediction =
                            engine.user_based_predict(k, user, item, method, chunks_opt);

                        match prediction {
                            Ok(predicted) => println!(
                                "Predicted score for item with id({}) is {}",
                                item_id, predicted
                            ),

                            Err(e) => {
                                log::error!("Failed to predict the score");
                                log::error!("Reason: {}", e);
                            }
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::ItemBasedPredict(
                        searchby_user,
                        searchby_item,
                        _method,
                        chunk_size,
                    ) => {
                        let user = match controller
                            .users_by(&searchby_user)
                            .map(|mut users| users.drain(..1).next().unwrap())
                        {
                            Ok(user) => user,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item = match controller
                            .items_by(&searchby_item)
                            .map(|mut items| items.drain(..1).next().unwrap())
                        {
                            Ok(item) => item,
                            Err(e) => {
                                log::error!("{}", e);
                                continue;
                            }
                        };

                        let item_id = item.get_id();

                        let now = Instant::now();
                        let prediction = engine.item_based_predict(user, item, chunk_size);

                        match prediction {
                            Ok(predicted) => println!(
                                "Predicted score for item with id({}) is {}",
                                item_id, predicted
                            ),

                            Err(e) => {
                                log::error!("Failed to predict the score");
                                log::error!("Reason: {}", e);
                            }
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::EnterSimMatrix(m, n, _method) => {
                        sim_matrix_prompt(&controller, name, m, n, rl)?;
                    }
                },

                None => log::error!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROMPT: &str = ">> ";

fn to_level_filter(level: usize) -> LevelFilter {
    match level {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    }
}

fn main() -> Result<(), Error> {
    let config = Config::load("config.toml")?;
    let term_level = to_level_filter(config.system.term_verbosity_level);
    let file_log = File::create("rs.log")?;
    let file_level = to_level_filter(config.system.file_verbosity_level);

    CombinedLogger::init(vec![
        TermLogger::new(term_level, LogConfig::default(), TerminalMode::Mixed),
        WriteLogger::new(file_level, LogConfig::default(), file_log),
    ])?;

    println!("Welcome to recommendation-system {}", VERSION);
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let opt: String = prompt!(rl)?;

        match opt.trim() {
            "q" | "quit" => {
                println!("Bye!");
                break;
            }

            "v" | "version" => {
                println!("version: {}", VERSION);
            }

            empty if empty.is_empty() => {}

            line => match parser::parse_line(line) {
                Some(stmt) => {
                    if let Statement::Connect(db) = stmt {
                        match db {
                            Database::Books => database_connected_prompt(
                                BooksController::new()?,
                                "books",
                                &mut rl,
                            )?,

                            Database::Shelves => database_connected_prompt(
                                ShelvesController::new()?,
                                "shelves",
                                &mut rl,
                            )?,

                            Database::SimpleMovie => database_connected_prompt(
                                SimpleMovieController::new()?,
                                "simple-movie",
                                &mut rl,
                            )?,

                            Database::MovieLens => database_connected_prompt(
                                MovieLensController::new()?,
                                "movie-lens",
                                &mut rl,
                            )?,

                            Database::MovieLensSmall => database_connected_prompt(
                                MovieLensSmallController::new()?,
                                "movie-lens-small",
                                &mut rl,
                            )?,
                        }
                    } else {
                        log::error!("Invalid statement in this context.");
                        log::error!("Connect to a database first!");
                    }
                }

                None => log::error!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}
