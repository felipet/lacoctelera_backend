// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    domain::{DataDomainError, Ingredient},
    routes::ingredient::utils::{check_ingredient, get_ingredient_from_db},
};
use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, error, info, instrument};
use utoipa::IntoParams;
use uuid::Uuid;

/// `Struct` QueryData models the expected fields for a query string.
///
/// # Description
///
/// Using a `Struct` rather than a simple `String` as the received data for the Query will leverage
/// the internal parsing logic of the framework. This way, the endpoint handler would only receive
/// valid data, since wrong data is rejected and the request is answered with a code 400 by the
/// framework.
#[derive(Deserialize, IntoParams)]
pub struct QueryData {
    pub name: String,
}

/// GET for the API's /ingredient endpoint.
#[utoipa::path(
    get,
    path = "/ingredient",
    tag = "Ingredient",
    params(
        QueryData
    ),
    responses(
        (
            status = 200,
            description = "The query was successfully executed",
            body = [Ingredient]
        ),
        (
            status = 400,
            description = "Error found in the given query",
        ),
    )
)]
#[instrument(
    skip(pool, req),
    fields(
        ingredient_name = %req.name,
    )
)]
#[get("")]
pub async fn search_ingredient(
    pool: Data<MySqlPool>,
    req: Query<QueryData>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // First, validate the given form as a correct name for the instantiation of an Ingredient.
    let query_ingredient = match Ingredient::parse(None, &req.name, "other", None) {
        Ok(ingredient) => {
            info!(
                "Received search request for an ingredient identified by: '{}'",
                ingredient.name()
            );
            ingredient
        }
        Err(e) => return Ok(HttpResponse::BadRequest().body(format!("{}", e))),
    };

    // Issue a query to the DB to search for ingredients using the given name.
    let ingredients = match check_ingredient(&pool, query_ingredient).await {
        Ok(ingredients) => {
            if !ingredients.is_empty() {
                let mut ing_list = String::new();
                ingredients
                    .iter()
                    .for_each(|i| ing_list.push_str(&format!("{{ {:#?} }},", i)));
                info!("Ingredients found: {}", ingredients.len());
                debug!("Ingredients found: {:#?}.", ing_list);
            } else {
                info!("No ingredients found.");
            }

            ingredients
        }
        Err(_) => Vec::new(),
    };

    Ok(HttpResponse::Ok().json(ingredients))
}

#[utoipa::path(
    get,
    context_path = "/ingredient/",
    tag = "Ingredient",
    responses(
        (
            status = 200,
            description = "The given ID matches an ingredient entry in the DB.",
            body = Ingredient,
            headers(
                ("Content-Length"),
                ("Content-Type"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
        ),
        (
            status = 404,
            description = "The given ingredient's ID was not found in the DB.",
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
    skip(pool, req),
    fields(
        ingredient_id = %req.0,
    )
)]
#[get("{id}")]
pub async fn get_ingredient(
    req: Path<(String,)>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let id = match Uuid::parse_str(&req.0) {
        Ok(id) => id,
        Err(e) => {
            error!("{e}");
            return Err(Box::new(DataDomainError::InvalidId));
        }
    };

    match get_ingredient_from_db(&pool, &id).await? {
        Some(ingredient) => Ok(HttpResponse::Ok().json(ingredient)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
