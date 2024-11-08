// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint DELETE method.

use crate::{
    authentication::{check_access, AuthData},
    domain::DataDomainError,
    routes::author::utils::delete_author_from_db,
};
use actix_web::{
    delete,
    web::{Data, Path, Query},
    HttpResponse,
};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{info, instrument};
use uuid::Uuid;

/// Delete an author from the system.
///
/// # Description
///
/// This method deletes an **Author** entry from the DB if the given ID matches the ID of a
/// registered author.
///
/// This method requires to provide a valid API token.
#[utoipa::path(
    delete,
    context_path = "/author/",
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, description = "The author was deleted from the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID didn't exist in the DB."),
    )
)]
#[instrument(skip(path, token, pool), fields(author_id = %path.0))]
#[delete("{id}")]
pub async fn delete_author(
    path: Path<(String,)>,
    token: Query<AuthData>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // Access control
    check_access(&pool, &token.api_key).await?;
    info!("Access granted");

    let author_id = match Uuid::parse_str(&path.0) {
        Ok(id) => id,
        Err(_) => return Err(Box::new(DataDomainError::InvalidId)),
    };

    delete_author_from_db(&pool, &author_id).await?;
    info!("Author {} deleted from the DB.", author_id.to_string());

    Ok(HttpResponse::Ok().finish())
}
