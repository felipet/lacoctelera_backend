// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
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
    #[error("The search criteria is invalid")]
    InvalidSearch,
    #[error("Expired access token")]
    ExpiredAccess,
    #[error("Wrong access token")]
    InvalidAccessCredentials,
    #[error("Email not registered in the DB")]
    InvalidEmail,
    #[error("Account disabled")]
    AccountDisabled,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Error from a DB query")]
    DbError,
    #[error("Error from the email client")]
    EmailClientError,
}

impl ResponseError for ServerError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ServerError::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::EmailClientError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::InternalServerError().body(format!(
            include_str!("../../static/message_template.html"),
            "<h3>Detected an error in the server, please, try again later.</h3>"
        ))
    }
}

impl ResponseError for DataDomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DataDomainError::InvalidAccessCredentials => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::InternalServerError().body(format!(
            include_str!("../../static/message_template.html"),
            "<h3>Detected an error in the server, please, try again later.</h3>"
        ))
    }
}
