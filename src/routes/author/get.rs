// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    authentication::{check_access, AuthData},
    domain::{AuthorBuilder, DataDomainError},
    routes::author::utils::{get_author_from_db, search_author_from_db},
};
use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, info, instrument};
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct QueryData {
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
}

impl QueryData {
    /// Returns the token that hash the highest priority in a search.
    ///
    /// # Description
    ///
    /// [QueryData] includes all the accepted tokens when a client requests a search of an author entry in the DB.
    /// All the tokens are marked as optional, to allow clients use the token they prefer. The current search logic
    /// only allows a single token search, which means that if multiple tokens are provided within the same request,
    /// only one will be considered.
    ///
    /// The **email** hash the highest priority, followed by **name** and **surname**. This method inspects what tokens
    /// where provided to the `struct`, and returns the one that hash the highest priority. If no token was provided,
    /// an error is returned instead.
    pub fn search_token(&self) -> Result<(&str, &str), DataDomainError> {
        if self.email.is_some() {
            Ok(("email", self.email.as_deref().unwrap()))
        } else if self.name.is_some() {
            Ok(("name", self.name.as_deref().unwrap()))
        } else if self.surname.is_some() {
            Ok(("surname", self.surname.as_deref().unwrap()))
        } else {
            info!("The given search params do not contain any valid token");
            Err(DataDomainError::InvalidSearch)
        }
    }
}

/// GET method for the Author endpoint.
///
/// # Description
///
/// This method searches an [Author] entry from the DB. If the author set the profile as non-public, only clients
/// with an API access token will retrieve the full author's descriptor. Unauthenticated clients will get the author's
/// name only when using this method of the endpoint.
#[utoipa::path(
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    params(QueryData),
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
#[instrument(skip(_token, pool))]
#[get("/author")]
pub async fn search_author(
    req: Query<QueryData>,
    _token: Option<Query<AuthData>>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("Search authors requested");
    let authors = search_author_from_db(&pool, req.0).await?;

    Ok(HttpResponse::Ok()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(authors))
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
