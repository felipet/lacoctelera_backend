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

/// Resource that allows to modify some of the attributes of an existing author in the DB.
///
/// # Description
///
/// This singleton resource of `/author` changes the content for the attributes given in the request body. The
/// resource accepts a JSON object, that defines (part of) an author entry of the DB. The bare minimum is to include
/// the author's ID, and an attribute to modify its content.
///
/// This resource requires the API client to provide an API token.
#[utoipa::path(
    patch,
    context_path = "/author/",
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = Author, description = "A partial definition of an Author entry.",
        example = json!({"id": "0191e13b-5ab7-78f1-bc06-be503a6c111b", "name": "Juana"})
    ),
    responses(
        (status = 200, description = "The author entry was updated in the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID didn't exist in the DB."),
    )
)]
#[instrument(skip(pool, token, path), fields(author_id = %path.0))]
#[patch("{id}")]
pub async fn patch_author(
    path: Path<(String,)>,
    req: Json<Author>,
    pool: Data<MySqlPool>,
    token: Query<AuthData>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // Access control
    check_access(&pool, &token.api_key).await?;
    debug!("Access granted");

    let author_id = &path.0;

    // First, get the current entry for the author identified by its ID.
    let mut existing_author = get_author_from_db(&pool, author_id).await?;
    existing_author.update_from(&req);
    debug!("Author modified: {:#?}", existing_author);
    modify_author_from_db(&pool, &existing_author).await?;
    info!("Author entry {author_id} modified");

    Ok(HttpResponse::Ok().finish())
}
