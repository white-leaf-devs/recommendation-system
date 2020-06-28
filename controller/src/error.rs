// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use thiserror::Error as DError;

#[derive(Debug, Clone, DError)]
pub enum ErrorKind {
    #[error("Couldn't found entity with id({0})")]
    NotFoundById(String),

    #[error("Couldn't found entity with name({0})")]
    NotFoundByName(String),

    #[error("Couldn't found entity with {0}({1})")]
    NotFoundByCustom(String, String),

    #[error("Controller function not implemented")]
    NotImplemented,

    #[error("Failed to cast bson value")]
    BsonConvert,

    #[error("Failed to cast to some value ({0})")]
    ValueConvert(String),

    #[error("Couldn't cast value to {0}")]
    CastingValue(&'static str),

    #[error("Couldn't update rating for user({0}) on item({1})")]
    UpdateRatingFailed(String, String),
    #[error("Couldn't insert rating for user({0}) on item({1})")]
    InsertRatingFailed(String, String),
}
