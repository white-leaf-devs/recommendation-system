// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use controller::{Field, Value};
use rustyline::Editor;
use std::collections::HashMap;

macro_rules! field {
    ($ed:ident, $name:expr, $opt:expr, $ty:expr) => {{
        use rustyline::error::ReadlineError;

        let msg = if $opt {
            format!("{}{} (optional, {}): ", $crate::PROMPT, $name, $ty)
        } else {
            format!("{}{} (required, {}): ", $crate::PROMPT, $name, $ty)
        };

        match $ed.readline(&msg) {
            Ok(line) => Ok(Some(line)),

            // CTRL-D
            Err(ReadlineError::Eof) => Ok(None),

            // Any error
            Err(e) => Err(e),
        }
    }};
}

pub(crate) fn build_prototype<'a>(
    rl: &mut Editor<()>,
    fields: Vec<Field<'a>>,
) -> Result<HashMap<&'a str, Value>, Error> {
    println!("Press CTRL-D to leave a field as 'empty'");
    let mut prototype = HashMap::new();

    for field in fields {
        let is_optional = field.is_optional();
        let (name, ty) = field.into_tuple();

        loop {
            let input: Option<String> = field!(rl, name, is_optional, ty)?;

            match input {
                Some(input) => {
                    let value = Value::from_str(&input, ty);
                    match value {
                        Ok(value) => {
                            prototype.insert(name, value);
                            break;
                        }

                        Err(e) => {
                            log::error!("Invalid value received!");
                            log::error!("Reason: {}", e);
                        }
                    }
                }

                None if is_optional => {
                    break;
                }

                None => {
                    log::error!("Field '{}' is required, cannot be empty!", name);
                    continue;
                }
            }
        }
    }

    Ok(prototype)
}
