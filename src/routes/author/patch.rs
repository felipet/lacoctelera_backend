//! Author endpoint PATCH method.

use actix_web::{patch, web, HttpResponse, Responder};

/// PATCH method for the Author endpoint (Restricted).
///
/// # Description
///
/// This method updates an [Author] entry in the DB if the given [AuthorId] matches the ID of a
/// registered author.
///
/// This method requires to authenticate the client using a valid [crate::AuthData::api_key].
#[utoipa::path(
    patch,
    tag = "Author",
    request_body(
        content = Author, description = "A partial definition of an Author entry.",
        example = json!({"id": "0191e13b-5ab7-78f1-bc06-be503a6c111b", "surname": "Doe"})
    ),
    responses(
        (status = 204, description = "The author entry was updated in the DB."),
        (status = 401, description = "The client has no access to this resource."),
        (status = 404, description = "An author identified by the given ID was not existing in the DB."),
    ),
    security(
        ("api_key" = [])
    )
)]
#[patch("/author/{id}")]
pub async fn patch_author(_path: web::Path<(String,)>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
