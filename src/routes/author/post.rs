// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::Author;
use actix_web::{post, web, HttpResponse};
use tracing::info;

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
    tag = "Author",
    security(
        ("api_key" = [])
    )
)]
#[post("/author")]
pub async fn post_author(req: web::Json<Author>) -> HttpResponse {
    info!("Post new recipe: {:#?}", req.0);

    HttpResponse::NotImplemented().finish()
}
