// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::fmt::{self, Display};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SearchBy {
    Id(String),
    Name(String),
    Custom(String, String),
}

impl SearchBy {
    pub fn id(id: &str) -> Self {
        Self::Id(id.into())
    }

    pub fn name(name: &str) -> Self {
        Self::Name(name.into())
    }

    pub fn custom(key: &str, val: &str) -> Self {
        Self::Custom(key.into(), val.into())
    }
}

impl Display for SearchBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchBy::Id(id) => write!(f, "id({})", id),
            SearchBy::Name(name) => write!(f, "name({})", name),
            SearchBy::Custom(key, val) => write!(f, "{}({})", key, val),
        }
    }
}
