use anyhow::Error;
use common_macros::hash_map;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimMatrixConfig {
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
    pub term_verbosity_level: usize,
    pub file_verbosity_level: usize,
    pub log_output: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Config {
    pub system: SystemConfig,
    pub engine: EngineConfig,
    pub sim_matrix: SimMatrixConfig,
    pub databases: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                term_verbosity_level: 0,
                file_verbosity_level: 3,
                log_output: Some("debugrs.log".to_string()),
            },
            engine: EngineConfig {
                partial_users_chunk_size: 10000,
            },
            sim_matrix: SimMatrixConfig {
                chunk_size_threshold: 0.3,
                partial_users_chunk_size: 10000,
                allow_chunk_optimization: true,
            },
            databases: hash_map! {
              "simple-movie".into() => "postgres://postgres:@localhost/simple-movie".into(),
              "books".into() => "postgres://postgres:@localhost/books".into(),
              "shelves".into() => "postgres://postgres:@localhost/shelves".into(),
              "movie-lens".into() => "postgres://postgres:@localhost/movie-lens".into(),
              "movie-lens-small".into() => "postgres://postgres:@localhost/movie-lens-small".into(),
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
                log_output: Some("rs.log".to_string()),
                term_verbosity_level: 1,
                file_verbosity_level: 2,
            },
            engine: EngineConfig {
                partial_users_chunk_size: 10000,
            },
            sim_matrix: SimMatrixConfig {
                chunk_size_threshold: 0.3,
                partial_users_chunk_size: 10000,
                allow_chunk_optimization: true,
            },
            databases: hash_map! {
                "some-database".into() => "postgres://postgres:@localhost/some-database".into()
            },
        };

        let loaded = Config::load("example.toml")?;
        assert_eq!(expected, loaded);

        Ok(())
    }
}
