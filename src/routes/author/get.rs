// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    authentication::{check_access, AuthData},
    domain::{Author, AuthorBuilder, DataDomainError},
    routes::author::utils::get_author_from_db,
};
use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, info, instrument};

/// GET method for the Author endpoint.
///
/// # Description
///
/// This method searches an [Author] entry from the DB. If the author set the profile as non-public, only clients
/// with an API access token will retrieve the full author's descriptor. Unauthenticated clients will get the author's
/// name only when using this method of the endpoint.
#[utoipa::path(
    get,
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    responses(
        (
            status = 200,
            description = "The given ID matches an existing author entry in the DB.",
            body = [Author],
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            ),
            examples(
                ("Existing author" = (
                    summary = "Returned JSON for an existing author",
                    value = json!(
                        AuthorBuilder::default()
                            .set_name("Jane")
                            .set_surname("Doe")
                            .set_email("jane_doe@mail.com")
                            .set_website("http://janedoe.com")
                            .set_shareable(true)
                            .build()
                            .unwrap()
                    )
                ))
            ),
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
                ("Retry-After", description = "Amount of time between requests (seconds).")
            )
        )
    )
)]
#[instrument]
#[get("/author")]
pub async fn search_author(
    req: Query<Author>,
    token: Query<AuthData>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("Author ID: {:#?} requested", req.0);
    info!("Sending default Author descriptor until the final logic is implemented.");

    let author = Author::default();

    Ok(HttpResponse::NotImplemented()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(author))
}

/// GET method for the Author endpoint.
///
/// # Description
///
/// This method retrieves an [Author] entry from the DB. If the author set the profile as non-public, only clients
/// with an API access token will retrieve the full author's descriptor. Unauthenticated clients will get the author's
/// name only when using this method of the endpoint.
#[utoipa::path(
    get,
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    responses(
        (
            status = 200,
            description = "The given ID matches an existing author entry in the DB.",
            body = [Author],
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            ),
            examples(
                ("Existing author" = (
                    summary = "Returned JSON for an existing author",
                    value = json!(
                        AuthorBuilder::default()
                            .set_name("Jane")
                            .set_surname("Doe")
                            .set_email("jane_doe@mail.com")
                            .set_website("http://janedoe.com")
                            .set_shareable(true)
                            .build()
                            .unwrap()
                    )
                ))
            ),
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
                ("Retry-After", description = "Amount of time between requests (seconds).")
            )
        )
    )
)]
#[instrument(skip(token, pool, path))]
#[get("/author/{id}")]
pub async fn get_author(
    path: Path<(String,)>,
    token: Option<Query<AuthData>>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let author_id = &path.0;
    info!("Data for the Author ({author_id}) requested");

    // First: does the author exists?
    let mut author = match get_author_from_db(&pool, author_id).await {
        Ok(author) => author,
        Err(e) => match e.downcast_ref() {
            Some(DataDomainError::InvalidId) => return Ok(HttpResponse::NotFound().finish()),
            _ => return Err(e),
        },
    };

    debug!("Author entry: {:?}", author);

    // Check if the client hash privileges to retrieve the full description of the Author.
    if token.is_some() {
        info!("The client included an API token to access the restricted resources");
        let token = token.unwrap().api_key.clone();
        check_access(&pool, token).await?;
        info!("Access granted");
    } else {
        info!("The client didn't include an API token to access the restricted resources. Private data will be muted.");
        if !author.shareable.unwrap() {
            author.mute_private_data();
        }
    }

    Ok(HttpResponse::Ok()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(author))
}
