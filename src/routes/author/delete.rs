//! Author endpoint DELETE method.

use crate::domain::Author;
use actix_web::{delete, web, HttpResponse, Responder};

/// DELETE method for the Author endpoint (Restricted).
///
/// # Description
///
/// This method deletes an [Author] entry from the DB if the given [AuthorId] matches the ID of a
/// registered author.
///
/// This method requires to authenticate the client using a valid [crate::AuthData::api_key].
#[utoipa::path(
    delete,
    tag = "Author",
    request_body(
        content = AuthorId, description = "ID of the author entry in the DB.",
        example = json!({"id": "0191e13b-5ab7-78f1-bc06-be503a6c111b"})
    ),
    responses(
        (status = 204, description = "The author was deleted from the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[delete("/author/{AuthorId}")]
pub async fn delete_author(_path: web::Path<(Author,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
