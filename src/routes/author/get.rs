use crate::domain::{Author, AuthorBuilder};
use actix_web::{get, web, HttpResponse, Responder};
use tracing::info;

/// GET method for the Author endpoint.
///
/// # Description
///
/// This method retrieves an [Author] entry from the DB. If the author set the profile as non-public, only clients
/// with an API access token will retrieve the full author's descriptor. Unauthenticated clients will get the author's
/// name only when using this method of the endpoint.
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
#[get("/author/{id}")]
pub async fn search_author(path: web::Path<(String,)>) -> impl Responder {
    info!("Author ID: {:#?} requested", path.0);
    info!("Sending default Author descriptor until the final logic is implemented.");

    let author = Author::default();

    HttpResponse::NotImplemented()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(author)
}

#[get("/author/{id}")]
pub async fn get_author(path: web::Path<(String,)>) -> impl Responder {
    info!("Author ID: {:#?} requested", path.0);
    info!("Sending default Author descriptor until the final logic is implemented.");

    let author = Author::default();

    HttpResponse::NotImplemented()
        // Store author's data in the cache for a day.
        .append_header(("Cache-Control", "max-age=86400"))
        .json(author)
}
