// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint PATCH method.

use crate::{
    authentication::{check_access, AuthData},
    domain::Author,
    routes::author::utils::{get_author_from_db, modify_author_from_db},
};
use actix_web::{
    patch,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, info, instrument};

/// PATCH method for the Author endpoint (Restricted).
///
/// # Description
///
/// This method updates an [Author] entry in the DB if the given [AuthorId] matches the ID of a
/// registered author.
///
/// This method requires to authenticate the client using a valid [crate::AuthData::api_key].
#[utoipa::path(
    patch,
    tag = "Author",
    request_body(
        content = Author, description = "A partial definition of an Author entry.",
        example = json!({"id": "0191e13b-5ab7-78f1-bc06-be503a6c111b", "surname": "Doe"})
    ),
    responses(
        (status = 204, description = "The author entry was updated in the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[instrument(skip(pool, token))]
#[patch("/author/{id}")]
pub async fn patch_author(
    path: Path<(String,)>,
    req: Json<Author>,
    pool: Data<MySqlPool>,
    token: Query<AuthData>,
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("Author PATCH request received");

    // // Access control
    let token = token.api_key.clone();
    check_access(&pool, token).await?;
    info!("Access granted");

    // Log the received payload
    debug!("Author entry: {:?}", req);
    let author_id = path.0.clone();

    // First, get the current entry for the author identified by its ID.
    let mut existing_author = get_author_from_db(&pool, &author_id).await?;
    existing_author.update_from(&req);
    info!("Author modified: {:#?}", existing_author);
    modify_author_from_db(&pool, &existing_author).await?;
    info!("Author entry {author_id} patched");

    Ok(HttpResponse::Accepted().finish())
}
