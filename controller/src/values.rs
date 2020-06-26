// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::error::ErrorKind;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    String,
    Bool,
    Int32,
    Int64,
    Double,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Field {
    Required(String, Type),
    Optional(String, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Double(f64),
}

impl Value {
    pub fn from_str(value: &str, tp: Type) -> Result<Self, ErrorKind> {
        let value = match tp {
            Type::String => Self::String(value.to_owned()),
            Type::Bool => {
                let value = match value {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => return Err(ErrorKind::ValueConvert("Invalid literal for bool".into())),
                };

                Self::Bool(value)
            }

            Type::Int32 => {
                let value: i32 = value
                    .parse()
                    .map_err(|e: <i32 as FromStr>::Err| ErrorKind::ValueConvert(e.to_string()))?;
                Self::Int32(value)
            }

            Type::Int64 => {
                let value: i64 = value
                    .parse()
                    .map_err(|e: <i64 as FromStr>::Err| ErrorKind::ValueConvert(e.to_string()))?;
                Self::Int64(value)
            }

            Type::Double => {
                let value: f64 = value
                    .parse()
                    .map_err(|e: <f64 as FromStr>::Err| ErrorKind::ValueConvert(e.to_string()))?;
                Self::Double(value)
            }
        };

        Ok(value)
    }

    pub fn as_string(&self) -> Result<&str, ErrorKind> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err(ErrorKind::CastingValue("String".into())),
        }
    }

    pub fn as_bool(&self) -> Result<bool, ErrorKind> {
        match self {
            Self::Bool(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("bool".into())),
        }
    }

    pub fn as_i32(&self) -> Result<i32, ErrorKind> {
        match self {
            Self::Int32(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("i32".into())),
        }
    }

    pub fn as_i64(&self) -> Result<i64, ErrorKind> {
        match self {
            Self::Int64(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("i64".into())),
        }
    }

    pub fn as_f64(&self) -> Result<f64, ErrorKind> {
        match self {
            Self::Double(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("f64".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;

    #[test]
    fn casting_string() -> Result<(), Error> {
        let value = Value::from_str("quebin31", Type::String)?;
        let value = value.as_string()?;

        assert_eq!(value, "quebin31");

        Ok(())
    }

    #[test]
    fn casting_bool() -> Result<(), Error> {
        let value = Value::from_str("true", Type::Bool)?;
        let value = value.as_bool()?;

        assert!(value);

        Ok(())
    }
}
