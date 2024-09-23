// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Example

use crate::domain::{QuantityUnit, Recipe, RecipeCategory, RecipeContains, RecipeQuery, Tag};
use actix_web::{get, web, HttpResponse, Responder};
use std::convert::TryFrom;
use std::fmt::Display;
use tracing::info;
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
#[get("/recipe")]
pub async fn search_recipe(req: web::Query<RecipeQuery>) -> impl Responder {
    let search_type: SearchType = (&req.0).try_into().expect("Wrong query");

    info!("Recipe search ({search_type}) using: {{{}}}", req.0);

    let template_recipe = Recipe::new(
        &Uuid::now_v7().to_string(),
        "Demo recipe",
        None,
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        &RecipeCategory::Easy.to_string(),
        Some("A delicious cocktail for summer."),
        None,
        &Vec::from([
            RecipeContains {
                quantity: 100.0,
                unit: QuantityUnit::Grams,
                ingredient_id: Uuid::now_v7(),
            },
            RecipeContains {
                quantity: 20.0,
                unit: QuantityUnit::MilliLiter,
                ingredient_id: Uuid::now_v7(),
            },
        ]),
        &["Pour all the ingredients in a shaker", "Shake and serve"],
        &Uuid::now_v7().to_string(),
    )
    .unwrap();

    HttpResponse::NotImplemented().json(template_recipe)
}

/// Retrieve a recipe from the DB using its unique ID.
#[utoipa::path(
    get,
    tag = "Recipe",
    responses(
        (
            status = 200,
            description = "The recipe identified by the given ID was found in the DB",
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
#[get("/recipe/{id}")]
pub async fn get_recipe(path: web::Path<(String,)>) -> impl Responder {
    info!("Recipe ID: {:#?} requested", path.0);
    info!("Sending default Recipe descriptor until the final logic is implemented.");

    let template_recipe = Recipe::new(
        &Uuid::now_v7().to_string(),
        "Demo recipe",
        None,
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        &RecipeCategory::Easy.to_string(),
        Some("A delicious cocktail for summer."),
        None,
        &Vec::from([
            RecipeContains {
                quantity: 100.0,
                unit: QuantityUnit::Grams,
                ingredient_id: Uuid::now_v7(),
            },
            RecipeContains {
                quantity: 20.0,
                unit: QuantityUnit::MilliLiter,
                ingredient_id: Uuid::now_v7(),
            },
        ]),
        &["Pour all the ingredients in a shaker", "Shake and serve"],
        &Uuid::now_v7().to_string(),
    )
    .unwrap();

    HttpResponse::NotImplemented().json(template_recipe)
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
