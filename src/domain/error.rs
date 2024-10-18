// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;
use validator::ValidationErrors;

/// Custom error type for the operations related to data domains's objects.
///
/// # Description
///
/// - [DataDomainError::InvalidParams] is returned when a data object is built using wrong data for some of its
///   members. This is a wrapper and contains the error messages that could have been generated by the internal logic.
/// - [DataDomainError::InvalidId] is returned when an object is built using an ID that is badly formatted.
#[derive(Error, Debug)]
pub enum DataDomainError {
    #[error("Some params contain an invalid format")]
    InvalidParams {
        #[from]
        source: ValidationErrors,
    },
    #[error("The given Author ID hash an invalid format")]
    InvalidId,
    #[error("The given string is not a valid recipe's category")]
    InvalidRecipeCategory,
    #[error("The data provided in the form is invalid")]
    InvalidFormData,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Error from a DB query")]
    DbError,
}

impl ResponseError for ServerError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ServerError::DbError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
