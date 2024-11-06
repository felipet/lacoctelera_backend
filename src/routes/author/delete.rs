// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint DELETE method.

use crate::{
    authentication::{check_access, AuthData},
    routes::author::utils::delete_author_from_db,
};
use actix_web::{delete, web, HttpResponse};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{info, instrument};
use uuid::Uuid;

/// DELETE method for the Author endpoint (Restricted).
///
/// # Description
///
/// This method deletes an [Author] entry from the DB if the given [AuthorId] matches the ID of a
/// registered author.
///
/// This method requires to authenticate the client using a valid [crate::AuthData::api_key].
#[utoipa::path(
    delete,
    path = "/author",
    tag = "Author",
    responses(
        (status = 204, description = "The author was deleted from the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[instrument]
#[delete("{id}")]
pub async fn delete_author(
    path: web::Path<(String,)>,
    token: web::Query<AuthData>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("Author DELETE request received");

    // // Access control
    let token = token.api_key.clone();
    check_access(&pool, token).await?;
    info!("Access granted");

    delete_author_from_db(&pool, &Uuid::parse_str(&path.0).unwrap()).await?;

    Ok(HttpResponse::Ok().finish())
}
