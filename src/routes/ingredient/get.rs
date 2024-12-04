// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::Ingredient;
use actix_web::{get, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{debug, info, instrument};
use utoipa::IntoParams;

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
    pool: web::Data<MySqlPool>,
    req: web::Query<QueryData>,
) -> impl Responder {
    // First, validate the given form as a correct name for the instantiation of an Ingredient.
    let query_ingredient = match Ingredient::parse(None, &req.name, "other", None) {
        Ok(ingredient) => {
            info!(
                "Received search request for an ingredient identified by: '{}'",
                ingredient.name()
            );
            ingredient
        }
        Err(e) => return HttpResponse::BadRequest().body(format!("{}", e)),
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

    HttpResponse::Ok().json(ingredients)
}

#[get("{id}")]
pub async fn get_ingredient(_req: web::Path<String>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[instrument(skip(pool, ingredient))]
async fn check_ingredient(
    pool: &MySqlPool,
    ingredient: Ingredient,
) -> Result<Vec<Ingredient>, Box<dyn Error>> {
    let rows = sqlx::query!(
        r#"SELECT `id`, `name`, `category`, `description` FROM Ingredient i WHERE i.name like ?"#,
        format!("%{}%", ingredient.name()),
    )
    .fetch_all(pool)
    .await?;

    let mut ingredients = Vec::new();
    for r in rows {
        ingredients.push(Ingredient::parse(
            Some(&r.id),
            r.name.as_str(),
            r.category.as_str(),
            r.description.as_deref(),
        )?);
    }

    Ok(ingredients)
}
