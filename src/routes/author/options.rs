//! Author endpoint OPTIONS method.

use crate::domain::Author;
use actix_web::{options, web, HttpResponse, Responder};

/// OPTIONS method for the Author endpoint (Public).
///
/// # Description
///
/// Returns the supported methods of the endpoint `/author/{id}`. Useful for preflight requests made by web browsers.
#[utoipa::path(
    options,
    tag = "Author",
    responses(
        (
            status = 200,
            headers(
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            )
        ),
    )
)]
#[options("/author/{AuthorId}")]
pub async fn options_author(_path: web::Path<(Author,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
