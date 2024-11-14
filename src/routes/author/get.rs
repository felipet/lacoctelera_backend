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

/// Object that includes the allowed tokens for a search of the `/author` resource.
///
/// # Description
///
/// All the members are optional, which means clients are free to choose what token to use for a search. The current
/// search logic of the `/author` collection resource allows only to use a single token per search. This means that if
/// multiple tokens are given, the one with the highest priority will be used.
/// The **email** hash the highest priority, followed by **name** and **surname**.
#[derive(Debug, Deserialize, IntoParams)]
pub struct AuthorQueryParams {
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
}

impl AuthorQueryParams {
    /// Returns the token that hash the highest priority in a search.
    ///
    /// # Description
    ///
    /// [AuthorQueryParams] includes all the accepted tokens when a client requests a search of an author entry in the DB.
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

/// Search recipe's authors either by email, name or surname.
///
/// # Description
///
/// This collection resource receives some search criteria via URL params, and performs a search in the DB to find
/// all the authors that match such criteria. Clients of the API with no API token would retrieve some author entries
/// with muted data. Authors specify whether their profiles are public or not. If a profile is not public, only
/// the authorised clients of the API (with a token) will get the whole profile information.
#[utoipa::path(
    tag = "Author",
    path = "/author",
    security(
        ("api_key" = [])
    ),
    params(AuthorQueryParams),
    responses(
        (
            status = 200,
            description = "Some author profiles were found using the given search criteria.",
            body = [Author],
            headers(
                ("Content-Length"),
                ("Content-Type"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
            examples(
                ("Existing author" = (
                    summary = "Returned JSON for an existing author",
                    value = json!([
                        AuthorBuilder::default()
                            .set_name("Jane")
                            .set_surname("Doe")
                            .set_email("jane_doe@mail.com")
                            .set_website("http://janedoe.com")
                            .set_shareable(true)
                            .build()
                            .unwrap()
                        ])
                ))
            ),
        ),
        (
            status = 404,
            description = "The given author's ID was not found in the DB.",
            headers(
                ("Content-Length"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
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
#[instrument(
    skip(token, pool, req),
    fields(
        author_email = %req.0.email.as_deref().unwrap_or_default(),
        author_name = %req.0.name.as_deref().unwrap_or_default(),
        author_surname =  %req.0.surname.as_deref().unwrap_or_default(),
    )
)]
#[get("")]
pub async fn search_author(
    req: Query<AuthorQueryParams>,
    token: Option<Query<AuthData>>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let mut authors = search_author_from_db(&pool, req.0).await?;

    debug!("Author descriptors found: {:?}", authors);

    // Access control
    let client_auth = match token {
        Some(token) => {
            debug!("The client included an API token to access the restricted resources.");
            check_access(&pool, &token.api_key).await?;
            debug!("Access granted");
            true
        }
        None => false,
    };

    if !client_auth {
        debug!("The client hash no API token to access the restricted resources. Private data will be muted.");
        authors.iter_mut().for_each(|e| e.mute_private_data());
    }

    Ok(HttpResponse::Ok().json(authors))
}

/// Retrieve an author descriptor using the author's ID.
///
/// # Description
///
/// This singleton resource allows clients of the API to retrieve the details of a recipe's author. Check out the
/// **Author** schema to obtain a detailed description of all the attributes of the Author object.
///
/// If the author sets the profile as non-public (_non-shareable_), only clients with an API access token will retrieve
/// the full author's descriptor. Unauthenticated clients will get the author's name, the personal website, and the
/// social profiles when that data was given to the system. Authors only are required to provide a valid email.
#[utoipa::path(
    get,
    context_path = "/author/",
    tag = "Author",
    security(
        ("api_key" = [])
    ),
    responses(
        (
            status = 200,
            description = "The Author descriptor was found using the given ID.",
            body = Author,
            headers(
                ("Content-Length"),
                ("Content-Type"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
            examples(
                ("Existing author" = (
                    summary = "Returned JSON for an existing author.",
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
                ("Content-Length"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
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
#[instrument(skip(token, pool, path), fields(author_id = %path.0))]
#[get("{id}")]
pub async fn get_author(
    path: Path<(String,)>,
    token: Option<Query<AuthData>>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // First: does the author exists?
    let author_id = &path.0;
    let mut author = match get_author_from_db(&pool, author_id).await {
        Ok(author) => author,
        Err(e) => match e.downcast_ref() {
            Some(DataDomainError::InvalidId) => return Ok(HttpResponse::NotFound().finish()),
            _ => return Err(e),
        },
    };

    debug!("Author descriptor found: {:?}", author);

    // Check if the client hash privileges to retrieve the full description of the Author.
    if token.is_some() {
        debug!("The client included an API token to access the restricted resources.");
        check_access(&pool, &token.unwrap().api_key).await?;
        debug!("Access granted");
    } else {
        debug!("The client hash no API token to access the restricted resources. Private data will be muted.");
        if !author.shareable() {
            author.mute_private_data();
        }
    }

    Ok(HttpResponse::Ok().json(author))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case(None, None, None, true, "")]
    #[case(Some("Jane"), None, None, false, "name")]
    #[case(None, Some("Doe"), None, false, "surname")]
    #[case(None, None, Some("jane@mail.com"), false, "email")]
    #[case(Some("Jane"), Some("Doe"), None, false, "name")]
    #[case(None, Some("Doe"), Some("jane@mail.com"), false, "email")]
    #[case(Some("Jane"), None, Some("jane@mail.com"), false, "email")]
    #[case(Some("Jane"), Some("Doe"), Some("jane@mail.com"), false, "email")]
    fn query_params(
        #[case] name: Option<&str>,
        #[case] surname: Option<&str>,
        #[case] email: Option<&str>,
        #[case] is_err: bool,
        #[case] expected_token: &str,
    ) {
        let query_params = AuthorQueryParams {
            name: name.map(String::from),
            surname: surname.map(String::from),
            email: email.map(String::from),
        };

        let token = query_params.search_token();
        assert_eq!(token.is_err(), is_err);
        if let Ok(token) = token {
            assert_eq!(token.0, expected_token);
        }
    }
}
