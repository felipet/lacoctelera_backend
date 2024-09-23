//! Recipe endpoint head method.

use crate::domain::RecipeQuery;
use actix_web::{head, web, HttpResponse, Responder};

/// HEAD method for the Recipe endpoint (Public).
///
/// # Description
///
/// This method checks the headers that a GET method to the endpoint `/recipe` would respond. This is useful to
/// check the header `Content-Length` and others without doing the full request.
#[utoipa::path(
    head,
    tag = "Recipe",
    responses(
        (
            status = 200,
            description = "The search query was successfully executed.",
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            )
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
#[head("/recipe")]
pub async fn head_recipe(_req: web::Query<RecipeQuery>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
