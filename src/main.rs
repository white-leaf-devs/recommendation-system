use anyhow::Error;
use books::BooksController;
use controller::{Controller, Entity};
use recommend::Engine;
use simple_movie::SimpleMovieController;
use text_io::read;

macro_rules! prompt {
    () => {{
        use std::io::{self, Write};

        print!("{}", PROMPT);
        io::stdout().lock().flush()
    }};

    ($db:expr) => {{
        use std::io::{self, Write};

        print!("({}) {}", $db, PROMPT);
        io::stdout().lock().flush()
    }};
}

pub enum Databases {
    Books,
    SimpleMovie,
    MovieLensSmall,
}

fn database_connected_prompt<C, U, I>(controller: C, name: &str) -> Result<(), Error>
where
    C: Controller<U, I>,
    U: Entity,
    I: Entity,
{
    let _engine = Engine::with_controller(controller);

    loop {
        prompt!(name)?;
        let opt: String = read!("{}\n");

        match opt.trim() {
            "q" | "quit" => {
                println!("Bye!");
                break;
            }

            "d" | "disconnect" => {
                println!("Disconnecting from database {}", name);
                break;
            }

            _compound => continue,
        }
    }

    Ok(())
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROMPT: &str = ">> ";

fn main() -> Result<(), Error> {
    println!("Welcome to recommendation-system {}", VERSION);
    println!();

    loop {
        prompt!()?;
        let opt: String = read!("{}\n");

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

            compound => {
                let splitted: Vec<_> = compound.split(' ').collect();
                match splitted[0] {
                    "c" | "connect" => {
                        println!("Connecting to database {}", &splitted[1]);
                        match splitted[1] {
                            "books" => database_connected_prompt(BooksController::new()?, "books")?,
                            "simple-movie" => database_connected_prompt(
                                SimpleMovieController::new()?,
                                "simple-movie",
                            )?,
                            unknown => {
                                println!("Unknown database {}", unknown);
                                continue;
                            }
                        }
                    }

                    wtf if wtf.is_empty() => println!(),

                    wtf => {
                        println!("Unknown command {}", wtf);
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}
