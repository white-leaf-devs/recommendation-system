pub mod parser;

use anyhow::Error;
use books::BooksController;
use controller::{Controller, Entity};
use movie_lens_small::MovieLensSmallController;
use parser::{Database, Index, Statement};
use recommend::{Engine, MapedDistance};
use simple_movie::SimpleMovieController;

macro_rules! prompt {
    () => {{
        let mut rl = rustyline::Editor::<()>::new();
        rl.readline(&format!("{}", PROMPT))
    }};

    ($db:expr) => {{
        let mut rl = rustyline::Editor::<()>::new();
        rl.readline(&format!("({}) {}", $db, PROMPT))
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

    loop {
        let opt: String = prompt!(name)?;

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
                            None => println!("Failed to calculate distance"),
                        }
                    }

                    Statement::KNN(index, k, method) => {
                        let user = get_user(&controller, &index);
                        let knn = user.and_then(|user| engine.knn(&user, k, method));

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

    loop {
        let opt: String = prompt!()?;

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
