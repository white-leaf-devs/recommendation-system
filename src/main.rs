pub mod parser;

use anyhow::Error;
use books::BooksController;
use controller::{Controller, Entity, ToTable};
use engine::{similarity_matrix::SimilarityMatrix, Engine};
use movie_lens::MovieLensController;
use movie_lens_small::MovieLensSmallController;
use parser::{Database, Statement};
use rustyline::Editor;
use shelves::ShelvesController;
use simple_movie::SimpleMovieController;
use std::{fmt::Display, hash::Hash, time::Instant};

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
    m: usize,
    n: usize,
    threshold: usize,
    rl: &mut Editor<()>,
) -> Result<(), Error>
where
    C: Controller<User, UserId, Item, ItemId>,
    User: Entity<Id = UserId> + ToTable,
    Item: Entity<Id = ItemId> + ToTable,
    UserId: Hash + Eq + Display + Clone,
    ItemId: Hash + Eq + Display + Clone,
{
    let mut sim_matrix = SimilarityMatrix::new(controller, m, n, threshold);
    let mut curr_i = 0;
    let mut curr_j = 0;
    let mut maybe_chunk = sim_matrix.get_chunk(curr_i, curr_j);

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
                                println!("{}", e);
                                continue;
                            }
                        };

                        let item_id_b = match controller.items_by(&searchby_b) {
                            Ok(items) => items[0].get_id(),
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        if let Some(chunk) = &maybe_chunk {
                            let val = chunk.get(&item_id_a).and_then(|row| row.get(&item_id_b));

                            if let Some(val) = val {
                                println!("Value for ({}, {}) is {}", item_id_a, item_id_b, val);
                            } else {
                                println!("No value found for ({}, {})", item_id_a, item_id_b);
                            }
                        } else {
                            println!("Failed to use current chunk (maybe out-of-bounds)");
                        }
                    }

                    Statement::SimMatrixMoveTo(i, j) => {
                        curr_i = i;
                        curr_j = j;
                        maybe_chunk = sim_matrix.get_chunk(curr_i, curr_j);
                    }

                    _ => {
                        println!("Invalid statement in this context.");
                        println!("Exit the matrix first!");
                    }
                },
                None => println!("Invalid syntax!"),
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
    User: Entity<Id = UserId> + ToTable,
    Item: Entity<Id = ItemId> + ToTable,
    UserId: Hash + Eq + Display + Clone,
    ItemId: Hash + Eq + Display + Clone,
{
    let engine = Engine::with_controller(&controller);

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
                        println!("Invalid statement in this context.");
                        println!("Disconnect from current database first!");
                    }

                    Statement::SimMatrixGet(_, _) | Statement::SimMatrixMoveTo(_, _) => {
                        println!("Invalid statement in this context.");
                        println!("Enter the matrix first!");
                    }

                    Statement::QueryUser(searchby) => match controller.users_by(&searchby) {
                        Ok(users) => {
                            for user in users {
                                println!("{}", user.to_table());
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::QueryItem(searchby) => match controller.items_by(&searchby) {
                        Ok(items) => {
                            for item in items {
                                println!("{}", item.to_table());
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::QueryRatings(searchby) => match controller.users_by(&searchby) {
                        Ok(users) => {
                            for user in users {
                                if let Ok(ratings) = controller.ratings_by(&user) {
                                    if !ratings.is_empty() {
                                        println!("{}", ratings.to_table());
                                    } else {
                                        println!(
                                            "No ratings found for user with id({})",
                                            user.get_id()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::UserDistance(searchby_a, searchby_b, method) => {
                        let users_a = match controller.users_by(&searchby_a) {
                            Ok(users) => users,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        let users_b = match controller.users_by(&searchby_b) {
                            Ok(users) => users,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        let now = Instant::now();
                        let dist = engine.user_distance(&users_a[0], &users_b[0], method);
                        match dist {
                            Some(dist) => println!("Distance is {}", dist),
                            None => println!("Distance couldn't be calculated or gave NaN/∞/-∞"),
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::UserKnn(k, searchby, method, chunks_opt) => {
                        let users = controller.users_by(&searchby);
                        let now = Instant::now();
                        let knn = match users {
                            Ok(users) => engine.user_knn(k, &users[0], method, chunks_opt),
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        let elapsed = now.elapsed().as_secs_f64();

                        match knn {
                            Some(knn) => {
                                if knn.is_empty() {
                                    println!("Couldn't found the {} nearest neighbours", k);
                                    println!("Try using a different metric");
                                    continue;
                                }

                                for (nn_id, dist) in knn {
                                    println!("Distance with user with id({}) is {}", nn_id, dist);
                                }
                            }

                            None => println!("Failed to calculate the {} nearest neighbors", k),
                        }

                        println!("Operation took {:.4} seconds", elapsed);
                    }

                    Statement::UserPredict(k, searchby_user, searchby_item, method, chunks_opt) => {
                        let users = match controller.users_by(&searchby_user) {
                            Ok(users) => users,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        let items = match controller.items_by(&searchby_item) {
                            Ok(items) => items,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        let now = Instant::now();
                        let prediction =
                            engine.user_predict(k, &users[0], &items[0], method, chunks_opt);
                        match prediction {
                            Some(predicted) => println!(
                                "Predicted score for item with id({}) is {}",
                                items[0].get_id(),
                                predicted
                            ),
                            None => {
                                println!("Failed to predict the score");
                                println!(
                                    "Try increasing 'k' or using a different metric for inner knn"
                                );
                            }
                        }

                        println!("Operation took {:.4} seconds", now.elapsed().as_secs_f64());
                    }

                    Statement::EnterSimMatrix(m, n, threshold, _method) => {
                        sim_matrix_prompt(&controller, name, m, n, threshold, rl)?;
                    }

                    _ => println!("Unimplemented statement"),
                },

                None => println!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROMPT: &str = ">> ";

fn main() -> Result<(), Error> {
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
                        println!("Invalid statement in this context.");
                        println!("Connect to a database first!");
                    }
                }
                None => println!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}
