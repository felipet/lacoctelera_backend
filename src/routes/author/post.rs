// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    authentication::{check_access, AuthData},
    domain::Author,
    routes::author::utils::register_new_author,
};
use actix_web::{
    post,
    web::{Data, Json, Query},
    HttpResponse,
};
use serde_json::json;
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, info, instrument};

/// POST method for the /author endpoint (Restricted)
///
/// # Description
///
/// This method creates a new author entry in the DB, which is described by the `Author` schema. When a new author
/// is aimed to be registered in the DB, only providing a valid email address is mandatory. A confirmation email will
/// be sent to that email, so unvalidated authors won't be able to register content in the DB. This is a measure to
/// avoid spamming content in the DB.
///
/// When an author registers without providing a name, a *funny name* will be assigned by the backend logic.
///
/// Authors are identified by an unique ID, thus there's no issue when the same names are registered multiple times.
#[utoipa::path(
    post,
    path = "/author",
    tag = "Author",
    security(
        ("api_key" = [])
    )
)]
#[instrument(skip(pool, token))]
#[post("")]
pub async fn post_author(
    req: Json<Author>,
    pool: Data<MySqlPool>,
    token: Query<AuthData>,
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("New author entry received");

    // Access control
    check_access(&pool, &token.api_key).await?;
    info!("Access granted");

    // Log the received payload
    debug!("Author entry: {:?}", req);

    // Store the received entry in the DB.
    let id = register_new_author(&pool, &req).await?;
    info!("New Author entry registered with id: {id}");

    Ok(HttpResponse::Accepted().json(json!({
        "id": id.to_string()
    })))
}
