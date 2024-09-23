//! Author endpoint DELETE method.

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
    responses(
        (status = 204, description = "The author was deleted from the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[delete("/author/{id}")]
pub async fn delete_author(_path: web::Path<(String,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
