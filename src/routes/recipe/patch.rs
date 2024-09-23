// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Author endpoint PATCH method.

use actix_web::{patch, web, HttpResponse, Responder};

/// PATCH method for the Recipe endpoint (Restricted).
///
/// # Description
///
/// This method updates an `Recipe` entry in the DB if the given `id` matches the ID of a
/// registered recipe.
///
/// This method requires to authenticate the client using a valid [crate::AuthData::api_key].
#[utoipa::path(
    patch,
    tag = "Recipe",
    request_body(
        content = Recipe, description = "A partial definition of an Recipe entry.",
        example = json!({"name": "The most delicious cocktail"})
    ),
    responses(
        (status = 204, description = "The recipe entry was updated in the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "A recipe identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[patch("/recipe/{id}")]
pub async fn patch_recipe(_path: web::Path<(String,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
