// Copyright (c) 2020 White Leaf
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use thiserror::Error as DError;

#[derive(Debug, Clone, DError)]
pub enum ErrorKind {
    #[error("Tried to divide by zero")]
    DivisionByZero,

    #[error("Indeterminate form 0/0")]
    IndeterminateForm,

    #[error("Empty ratings")]
    EmptyRatings,

    #[error("Couldn't get distance, no matching ratings")]
    NoMatchingRatings,

    #[error("Couldn't convert types")]
    ConvertType,

    #[error("Empty k nearest neighbors")]
    EmptyKNearestNeighbors,

    #[error("This feature is not implemented yet")]
    NotImplemented,

    #[error("Indices out of bounds")]
    IndexOutOfBound,
}
