// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint head method.

use crate::{domain::DataDomainError, routes::author::utils::get_author_from_db};
use actix_web::{
    head,
    web::{Data, Path},
    HttpResponse,
};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::instrument;

/// Metadata request for an author.
#[utoipa::path(
    head,
    context_path = "/author/",
    tag = "Author",
    responses(
        (
            status = 200,
            description = "The given ID matches an existing author entry in the DB.",
            headers(
                ("Content-Length"),
                ("Content-Type"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            )
        ),
        (
            status = 404,
            description = "The given author's ID was not found in the DB.",
            headers(
                ("Content-Length"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
        ),
        (
            status = 429, description = "**Too many requests.**",
            headers(
                ("Cache-Control", description = "Cache control is set to *no-cache*."),
                ("Access-Control-Allow-Origin"),
                ("Retry-After", description = "Amount of time between requests (seconds)."),
            )
        )
    )
)]
#[instrument(skip(pool, path), fields(author_id = %path.0))]
#[head("{id}")]
pub async fn head_author(
    path: Path<(String,)>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let author_id = &path.0;
    // First: does the author exists?
    let author = match get_author_from_db(&pool, author_id).await {
        Ok(author) => author,
        Err(e) => match e.downcast_ref() {
            Some(DataDomainError::InvalidId) => return Ok(HttpResponse::NotFound().finish()),
            _ => return Err(e),
        },
    };

    Ok(HttpResponse::Ok().json(author))
}
