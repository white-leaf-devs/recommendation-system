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
    Int16,
    Int32,
    Int64,
    Double,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Field<'a> {
    Required(&'a str, Type),
    Optional(&'a str, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Int16(i16),
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

            Type::Int16 => {
                let value: i16 = value
                    .parse()
                    .map_err(|e: <i16 as FromStr>::Err| ErrorKind::ValueConvert(e.to_string()))?;
                Self::Int16(value)
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
            _ => Err(ErrorKind::CastingValue("String")),
        }
    }

    pub fn as_bool(&self) -> Result<bool, ErrorKind> {
        match self {
            Self::Bool(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("bool")),
        }
    }

    pub fn as_i16(&self) -> Result<i16, ErrorKind> {
        match self {
            Self::Int16(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("i16")),
        }
    }

    pub fn as_i32(&self) -> Result<i32, ErrorKind> {
        match self {
            Self::Int32(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("i32")),
        }
    }

    pub fn as_i64(&self) -> Result<i64, ErrorKind> {
        match self {
            Self::Int64(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("i64")),
        }
    }

    pub fn as_f64(&self) -> Result<f64, ErrorKind> {
        match self {
            Self::Double(v) => Ok(*v),
            _ => Err(ErrorKind::CastingValue("f64")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use assert_approx_eq::*;

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

    #[test]
    fn casting_i16() -> Result<(), Error> {
        let value = Value::from_str("123", Type::Int16)?;
        let value = value.as_i16()?;

        assert_eq!(value, 123);

        Ok(())
    }

    #[test]
    fn casting_i32() -> Result<(), Error> {
        let value = Value::from_str("1234", Type::Int32)?;
        let value = value.as_i32()?;

        assert_eq!(value, 1234);

        Ok(())
    }

    #[test]
    fn casting_i64() -> Result<(), Error> {
        let value = Value::from_str("1234", Type::Int64)?;
        let value = value.as_i64()?;

        assert_eq!(value, 1234);

        Ok(())
    }

    #[test]
    fn casting_f64() -> Result<(), Error> {
        let value = Value::from_str("1234.12", Type::Double)?;
        let value = value.as_f64()?;

        assert_approx_eq!(value, 1234.12);

        Ok(())
    }
}
