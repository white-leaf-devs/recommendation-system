// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::Error;
use common_macros::hash_map;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DatabaseEntry {
    pub psql_url: String,
    pub mongo_url: String,
    pub mongo_db: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MatrixConfig {
    pub chunk_size_threshold: f64,
    pub partial_users_chunk_size: usize,
    pub allow_chunk_optimization: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EngineConfig {
    pub partial_users_chunk_size: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SystemConfig {
    pub use_postgres: bool,
    pub term_verbosity_level: usize,
    pub file_verbosity_level: usize,
    pub log_output: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Config {
    pub system: SystemConfig,
    pub engine: EngineConfig,
    pub matrix: MatrixConfig,
    pub databases: HashMap<String, DatabaseEntry>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                use_postgres: false,
                term_verbosity_level: 0,
                file_verbosity_level: 3,
                log_output: Some("debugrs.log".to_string()),
            },
            engine: EngineConfig {
                partial_users_chunk_size: 10000,
            },
            matrix: MatrixConfig {
                chunk_size_threshold: 0.3,
                partial_users_chunk_size: 10000,
                allow_chunk_optimization: true,
            },
            databases: hash_map! {
                "simple-movie".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/simple-movie".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "simple-movie".into()
                },
                "books".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/books".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "books".into()
                },
                "shelves".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/shelves".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "shelves".into(),
                },
                "movie-lens".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/movie-lens".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "movie-lens".into(),
                },
                "movie-lens-small".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/movie-lens-small".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "movie-lens-small".into(),
                }
            },
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        let parsed: Self = toml::from_str(&contents)?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use common_macros::hash_map;

    #[test]
    fn load_example_config() -> Result<(), Error> {
        let expected = Config {
            system: SystemConfig {
                use_postgres: false,
                log_output: Some("rs.log".to_string()),
                term_verbosity_level: 1,
                file_verbosity_level: 2,
            },
            engine: EngineConfig {
                partial_users_chunk_size: 10000,
            },
            matrix: MatrixConfig {
                chunk_size_threshold: 0.3,
                partial_users_chunk_size: 10000,
                allow_chunk_optimization: true,
            },
            databases: hash_map! {
                "some-database".into() => DatabaseEntry {
                    psql_url: "postgres://postgres:@localhost/some-database".into(),
                    mongo_url: "mongodb://localhost:27017".into(),
                    mongo_db: "some-database".into(),
                }
            },
        };

        let loaded = Config::load("example.toml")?;
        assert_eq!(expected, loaded);

        Ok(())
    }
}
