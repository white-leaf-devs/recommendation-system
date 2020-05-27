pub mod parser;

use anyhow::Error;
use books::BooksController;
use controller::{ratings_to_table, Controller, Entity};
use movie_lens_small::MovieLensSmallController;
use parser::{Database, Index, Statement};
use recommend::{Engine, MapedDistance};
use simple_movie::SimpleMovieController;

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
                println!("Bye!");
                break;
            }

            Err(e) => Err(e),
        }
    }};
}

fn get_user<C, U, I>(controller: &C, index: &Index) -> Option<U>
where
    C: Controller<U, I>,
    U: Entity,
    I: Entity,
{
    match index {
        Index::Id(id) => match controller.user_by_id(&id) {
            Ok(user) => Some(user),
            Err(e) => {
                println!("{}", e);
                None
            }
        },
        Index::Name(name) => match controller.user_by_name(&name) {
            Ok(users) => {
                if let Some(user) = users.into_iter().next() {
                    Some(user)
                } else {
                    println!("Failed to find by name, empty");
                    None
                }
            }
            Err(e) => {
                println!("{}", e);
                None
            }
        },
    }
}

fn get_item<C, U, I>(controller: &C, index: &Index) -> Option<I>
where
    C: Controller<U, I>,
    U: Entity,
    I: Entity,
{
    match index {
        Index::Id(id) => match controller.item_by_id(&id) {
            Ok(item) => Some(item),
            Err(e) => {
                println!("{}", e);
                None
            }
        },
        Index::Name(name) => match controller.item_by_name(&name) {
            Ok(items) => {
                if let Some(item) = items.into_iter().next() {
                    Some(item)
                } else {
                    println!("Failed to find by name, empty");
                    None
                }
            }

            Err(e) => {
                println!("{}", e);
                None
            }
        },
    }
}

fn database_connected_prompt<C, U, I>(controller: C, name: &str) -> Result<(), Error>
where
    C: Controller<U, I>,
    U: Entity,
    I: Entity,
{
    let engine = Engine::with_controller(&controller);
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let opt: String = prompt!(rl, name)?;

        match opt.trim() {
            "q" | "quit" => {
                println!("Bye!");
                break;
            }

            "d" | "disconnect" => {
                println!("Disconnecting from database {}", name);
                break;
            }

            line => match parser::parse_line(line) {
                Some(stmt) => match stmt {
                    Statement::Connect(_) => println!("Invalid in this context!"),

                    Statement::QueryUser(index) => match index {
                        Index::Id(id) => match controller.user_by_id(&id) {
                            Ok(user) => println!("{}", user.to_table()),
                            Err(e) => println!("{}", e),
                        },
                        Index::Name(name) => match controller.user_by_name(&name) {
                            Ok(users) => {
                                if users.is_empty() {
                                    println!("Not found, empty result");
                                    continue;
                                }

                                for user in users {
                                    println!("{}", user.to_table());
                                }
                            }
                            Err(e) => println!("{}", e),
                        },
                    },

                    Statement::QueryItem(index) => match index {
                        Index::Id(id) => match controller.item_by_id(&id) {
                            Ok(item) => println!("{}", item.to_table()),
                            Err(e) => println!("{}", e),
                        },
                        Index::Name(name) => match controller.item_by_name(&name) {
                            Ok(items) => {
                                if items.is_empty() {
                                    println!("Not found, empty result");
                                    continue;
                                }

                                for item in items {
                                    println!("{}", item.to_table());
                                }
                            }
                            Err(e) => println!("{}", e),
                        },
                    },

                    Statement::QueryRatings(index) => {
                        let ratings = get_user(&controller, &index)
                            .map(|user| controller.ratings_by_user(&user));

                        match ratings {
                            Some(Ok(ratings)) => {
                                if ratings.is_empty() {
                                    println!("Result is empty");
                                    continue;
                                }

                                println!("{}", ratings_to_table(&ratings));
                            }
                            Some(Err(e)) => println!("{}", e),
                            _ => println!("Not found, empty result"),
                        }
                    }

                    Statement::Distance(index_a, index_b, method) => {
                        let user_a = get_user(&controller, &index_a);
                        let user_b = get_user(&controller, &index_b);

                        let dist = match (user_a, user_b) {
                            (Some(user_a), Some(user_b)) => {
                                engine.distance(&user_a, &user_b, method)
                            }
                            (_, _) => None,
                        };

                        match dist {
                            Some(dist) => println!("Distance is {}", dist),
                            None => println!("Failed to calculate distance, maybe one is missing"),
                        }
                    }

                    Statement::KNN(k, index, method) => {
                        let knn = get_user(&controller, &index)
                            .and_then(|user| engine.knn(k, &user, method));

                        match knn {
                            Some(knn) => {
                                if knn.is_empty() {
                                    println!("Empty result");
                                    continue;
                                }

                                for MapedDistance(nn_id, dist) in knn {
                                    println!("Distance with id({}) is {}", nn_id.0, dist);
                                }
                            }
                            None => println!("Failed to calculate distance"),
                        }
                    }

                    Statement::Predict(k, index_user, index_item, method) => {
                        let user = get_user(&controller, &index_user);
                        let item = get_item(&controller, &index_item);

                        let prediction = match (user, item) {
                            (Some(user), Some(item)) => engine.predict(k, &user, &item, method),
                            _ => None,
                        };

                        match prediction {
                            Some(predicted) => println!("Predicted value is {}", predicted),
                            None => println!("Failed to predict, maybe one is missing"),
                        }
                    }
                },

                None => println!("Invalid syntax"),
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
            "?" | "h" | "help" => {
                println!("Main help:");
                println!("h | help           Shows this help");
                println!("q | quit           Quit");
                println!("c | connect <DB>   Connect to DB");
            }

            "q" | "quit" => {
                println!("Bye!");
                break;
            }

            empty if empty.is_empty() => {}

            line => match parser::parse_line(line) {
                Some(stmt) => {
                    if let Statement::Connect(db) = stmt {
                        match db {
                            Database::Books => {
                                database_connected_prompt(BooksController::new()?, "books")?
                            }
                            Database::SimpleMovie => database_connected_prompt(
                                SimpleMovieController::new()?,
                                "simple-movie",
                            )?,
                            Database::MovieLensSmall => database_connected_prompt(
                                MovieLensSmallController::new()?,
                                "movie-lens-small",
                            )?,
                        }
                    } else {
                        println!("Invalid statement in this context!");
                    }
                }
                None => println!("Invalid syntax!"),
            },
        }
    }

    Ok(())
}
