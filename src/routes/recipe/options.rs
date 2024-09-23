// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Recipe endpoint OPTIONS method.

use actix_web::{options, HttpResponse, Responder};

/// OPTIONS method for the Recipe endpoint (Public).
///
/// # Description
///
/// Returns the supported methods of the endpoint `/recipe`. Useful for preflight requests made by web browsers.
#[utoipa::path(
    options,
    tag = "Recipe",
    responses(
        (
            status = 200,
            headers(
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            )
        ),
    )
)]
#[options("/recipe")]
pub async fn options_recipe() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
