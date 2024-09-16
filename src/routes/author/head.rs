//! Author endpoint head method.

use crate::domain::Author;
use actix_web::{head, web, HttpResponse, Responder};

/// HEAD method for the Author endpoint (Public).
///
/// # Description
///
/// This method checks the headers that a GET method to the endpoint `/author/{id}` would respond. This is useful to
/// check the header `Content-Length` and others without doing the full request.
#[utoipa::path(
    head,
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
#[head("/author/{AuthorId}")]
pub async fn head_author(_path: web::Path<(Author,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
