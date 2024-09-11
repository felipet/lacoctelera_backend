use crate::domain::{author::AuthorBuilder, Author};
use actix_web::{get, web, HttpResponse, Responder};
use core::fmt;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[into_params(names("AuthorId"))]
#[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
pub struct AuthorId(Uuid);

impl fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[utoipa::path(
    get,
    tag = "Author",
    responses(
        (
            status = 200,
            description = "The given ID matches an existing author entry in the DB.",
            body = [Author],
            headers(
                ("Cache-Control"),
                ("Access-Control-Allow-Origin"),
                ("Content-Type")
            ),
            examples(
                ("Existing author" = (
                    summary = "Returned JSON for an existing author",
                    value = json!(
                        AuthorBuilder::default()
                            .set_name("Jane")
                            .set_surname("Doe")
                            .set_email("jane_doe@mail.com")
                            .set_website("http://janedoe.com")
                            .set_shareable(true)
                            .build()
                            .unwrap()
                    )
                ))
            ),
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
                ("Retry-After", description = "Amount of time between requests (seconds).")
            )
        )
    )
)]
#[get("/author/{AuthorId}")]
pub async fn get_author(path: web::Path<(AuthorId,)>) -> impl Responder {
    info!("Author ID: {} requested", path.0);
    info!("Sending default Author descriptor until the final logic is implemented.");

    let author = Author::default();

    HttpResponse::NotImplemented()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(author)
}
