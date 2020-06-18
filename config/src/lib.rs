use anyhow::Error;
use serde::Deserialize;
use std::path::Path;

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
pub struct Config {
    pub engine: EngineConfig,
    pub sim_matrix: SimMatrixConfig,
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

    #[test]
    fn load_example_config() -> Result<(), Error> {
        let expected = Config {
            engine: EngineConfig {
                partial_users_chunk_size: 10000,
            },
            sim_matrix: SimMatrixConfig {
                chunk_size_threshold: 0.3,
                partial_users_chunk_size: 10000,
                allow_chunk_optimization: true,
            },
        };

        let loaded = Config::load("example.toml")?;
        assert_eq!(expected, loaded);

        Ok(())
    }
}
