pub mod parser;

use anyhow::Error;
use books::BooksController;
use controller::{Controller, Entity, ToTable};
use movie_lens_small::MovieLensSmallController;
use parser::{Database, Statement};
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

            "v" | "version" => {
                println!("version: {}", VERSION);
            }

            line => match parser::parse_line(line) {
                Some(stmt) => match stmt {
                    Statement::Connect(_) => println!("Invalid in this context!"),

                    Statement::QueryUser(searchby) => match controller.users(&searchby) {
                        Ok(users) => {
                            for user in users {
                                println!("{}", user.to_table());
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::QueryItem(searchby) => match controller.items(&searchby) {
                        Ok(items) => {
                            for item in items {
                                println!("{}", item.to_table());
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::QueryRatings(searchby) => match controller.users(&searchby) {
                        Ok(users) => {
                            for user in users {
                                if let Ok(ratings) = controller.ratings_by(&user) {
                                    if !ratings.is_empty() {
                                        println!("{}", ratings.to_table());
                                    } else {
                                        println!("No ratings found for id({})", user.get_id());
                                    }
                                }
                            }
                        }
                        Err(e) => println!("{}", e),
                    },

                    Statement::Distance(searchby_a, searchby_b, method) => {
                        let users_a = controller.users(&searchby_a);
                        let users_b = controller.users(&searchby_b);

                        let dist = match (users_a, users_b) {
                            (Ok(users_a), Ok(users_b)) => {
                                engine.distance(&users_a[0], &users_b[0], method)
                            }
                            (_, _) => None,
                        };

                        match dist {
                            Some(dist) => println!("Distance is {}", dist),
                            None => println!("Failed to calculate distance, maybe one is missing"),
                        }
                    }

                    Statement::KNN(k, searchby, method) => {
                        let users = controller.users(&searchby);
                        let knn = match users {
                            Ok(users) => engine.knn(k, &users[0], method),

                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        };

                        match knn {
                            Some(knn) => {
                                if knn.is_empty() {
                                    println!("Couldn't found {} neighbours", k);
                                    continue;
                                }

                                for MapedDistance(nn_id, dist) in knn {
                                    println!("Distance with id({}) is {}", nn_id, dist);
                                }
                            }

                            None => println!("Failed to calculate {} nearest neighbors", k),
                        }
                    }

                    Statement::Predict(k, searchby_user, searchby_item, method) => {
                        let users = controller.users(&searchby_user);
                        let items = controller.items(&searchby_item);

                        let prediction = match (users, items) {
                            (Ok(users), Ok(items)) => {
                                engine.predict(k, &users[0], &items[0], method)
                            }

                            _ => None,
                        };

                        match prediction {
                            Some(predicted) => println!("Predicted value is {}", predicted),
                            None => println!("Failed to predict an score"),
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

            "v" | "version" => {
                println!("version: {}", VERSION);
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
