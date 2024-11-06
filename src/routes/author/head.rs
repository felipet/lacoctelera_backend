// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint head method.

use actix_web::{head, web, HttpResponse, Responder};

/// HEAD method for the Author endpoint (Public).
///
/// # Description
///
/// This method checks the headers that a GET method to the endpoint `/author/{id}` would respond. This is useful to
/// check the header `Content-Length` and others without doing the full request.
#[utoipa::path(
    head,
    path = "/author",
    tag = "Author",
    responses(
        (
            status = 200,
            description = "The given ID matches an existing author entry in the DB.",
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            )
        ),
        (
            status = 404,
            description = "The given author's ID was not found in the DB.",
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
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
#[head("{id}")]
pub async fn head_author(_path: web::Path<(String,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
