// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Example

use crate::{
    domain::{DataDomainError, RecipeQuery},
    routes::recipe::{
        get_recipe_from_db, search_recipe_by_category, search_recipe_by_name,
        search_recipe_by_rating,
    },
};
use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use sqlx::MySqlPool;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;
use tracing::{info, instrument};
use uuid::Uuid;

/// GET method for the /recipe endpoint (Public).
///
/// # Description
///
/// The GET method allows *searching* a recipe in the DB. It expects multiple attributes to filter the recipes in the
/// DB that shall be encoded in the url. The following keys can be used to perform a search:
/// - `name`: Use a string that can match the name of a recipe (or part of it).
/// - `tags`: Only recipes that contain all the included tags in the query will be returned by the API.
/// - `rating`: Recipes that are scored with a rating greater or equal to the given rating will be returned by the API.
///   See the schema `RecipeRating` for more details.
/// - `category`: Filter recipes using one of the available categories. See the schema `RecipeCategory` for more
///    details.
///
/// A query can be composed by many attributes. For example, consider this query:
///
/// ```bash
/// http://localhost:9090/recipe?name=margarita&tags=tequila&tags=reposado&rating=2
/// ```
///
/// Would return recipes that contain the string *margarita* in their name attribute; whose tags include *tequila* and
/// *reposado*; and, whose rating is greater or equal to 4 stars.
#[utoipa::path(
    get,
    path = "/recipe",
    tag = "Recipe",
    params(RecipeQuery),
    responses(
        (
            status = 200,
            description = "The query was executed successfully",
            body = [Recipe],
            headers(
                ("Access-Control-Allow-Origin"),
                ("Content-Type"),
                ("Cache-Control"),
            )
        ),
        (
            status = 429,
            description = "Too many requests",
            headers(
                ("Access-Control-Allow-Origin"),
                ("Retry-After"),
            )
        ),

    )
)]
#[get("")]
pub async fn search_recipe(
    req: Query<RecipeQuery>,
    pool: Data<MySqlPool>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let search_type: SearchType = (&req.0).try_into().expect("Wrong query");

    info!("Recipe search ({search_type}) using: {{{}}}", req.0);

    let recipe_ids = match search_type {
        SearchType::ByName => {
            let search_token = match req.0.name {
                Some(name) => name,
                None => return Err(Box::new(DataDomainError::InvalidSearch)),
            };
            search_recipe_by_name(&pool, &search_token).await?
        }
        SearchType::ByCategory => {
            let search_token = match req.0.category {
                Some(category) => category,
                None => return Err(Box::new(DataDomainError::InvalidSearch)),
            };
            search_recipe_by_category(&pool, search_token).await?
        }
        SearchType::ByRating => {
            let search_token = match req.0.rating {
                Some(rating) => rating,
                None => return Err(Box::new(DataDomainError::InvalidSearch)),
            };
            search_recipe_by_rating(&pool, search_token).await?
        }
        SearchType::ByTags => return Ok(HttpResponse::NotImplemented().finish()),
        SearchType::Intersection => return Ok(HttpResponse::NotImplemented().finish()),
    };

    let mut recipes = Vec::new();

    for id in recipe_ids.iter() {
        recipes.push(get_recipe_from_db(&pool, id).await?)
    }

    Ok(HttpResponse::Ok().json(recipes))
}

/// Retrieve a recipe from the DB using its unique ID.
#[utoipa::path(
    get,
    context_path = "/recipe/",
    tag = "Recipe",
    responses(
        (
            status = 200,
            description = "The recipe identified by the given ID was found in the DB",
            body = Recipe,
            headers(
                ("Content-Length"),
                ("Content-Type"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
        ),
        (
            status = 404,
            description = "The given recipe's ID was not found in the DB.",
            headers(
                ("Content-Length"),
                ("Date"),
                ("Vary", description = "Origin,Access-Control-Request-Method,Access-Control-Request-Headers")
            ),
        ),
        (
            status = 429,
            description = "Too many requests",
            headers(
                ("Access-Control-Allow-Origin"),
                ("Retry-After"),
            )
        ),

    )

)]
#[instrument(skip(pool))]
#[get("{id}")]
pub async fn get_recipe(
    pool: Data<MySqlPool>,
    path: Path<(String,)>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let recipe_id = Uuid::parse_str(&path.0).map_err(|_| DataDomainError::InvalidId)?;

    let recipe = get_recipe_from_db(&pool, &recipe_id).await?;

    Ok(HttpResponse::Ok().json(recipe))
}

#[derive(Debug, Clone)]
enum SearchType {
    ByName,
    ByTags,
    ByRating,
    ByCategory,
    Intersection,
}

impl Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ss = match self {
            SearchType::ByName => "ByName",
            SearchType::ByTags => "ByTags",
            SearchType::ByRating => "ByRating",
            SearchType::ByCategory => "ByCategory",
            SearchType::Intersection => "Intersection",
        };

        write!(f, "{ss}")
    }
}

fn multiple_choices(query: &RecipeQuery) -> bool {
    if (query.name.is_some()
        && (query.tags.is_some() || query.rating.is_some() || query.category.is_some()))
        || (query.tags.is_some() && (query.rating.is_some() || query.category.is_some()))
        || (query.rating.is_some() && query.category.is_some())
    {
        return true;
    }

    false
}

impl TryFrom<&RecipeQuery> for SearchType {
    type Error = String;

    fn try_from(query: &RecipeQuery) -> std::result::Result<Self, Self::Error> {
        if multiple_choices(query) {
            Ok(SearchType::Intersection)
        } else if query.name.is_some() {
            Ok(SearchType::ByName)
        } else if query.tags.is_some() {
            Ok(SearchType::ByTags)
        } else if query.rating.is_some() {
            Ok(SearchType::ByRating)
        } else if query.category.is_some() {
            Ok(SearchType::ByCategory)
        } else {
            Err("Invalid conversion".to_string())
        }
    }
}
