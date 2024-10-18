//! Request a new API token for the restricted endpoints.

use crate::domain::auth::TokenRequestData;
use actix_web::{
    get, http::header::ContentType, post, web::Data, web::Form, HttpRequest, HttpResponse,
    Responder,
};
use sqlx::MySqlPool;
use tracing::{debug, error, info};

/// GET for the API's /token/request endpoint.
///
/// # Description
///
/// This endpoint offers a simple HTML form that allows clients interested in accessing the restricted endpoints to
/// request an API token.
#[utoipa::path(
    tag = "Token",
    responses(
        (
            status = 200,
            description = "A simple HTML page with a form."
        )
    )
)]
#[get("/request")]
pub async fn token_req_get() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("token_request.html"))
}

#[post("/request")]
pub async fn token_req_post(req: HttpRequest, form: Form<TokenRequestData>) -> impl Responder {
    let mut message_html = String::new();

    let request = TokenRequestData::new(form.0.email(), form.0.explanation());

    if request.is_ok() {
        message_html.insert_str(0, "<h3>The request was sent. After the request is evaluated, an email will be sent to inform you about the following steps.</h3>");
        let request = request.unwrap();
        info!("An API token was requested by {}", request.email());

        return HttpResponse::NotImplemented().finish();
    } else {
        debug!(
            "Failed attempt to request an API token from the IP: {}",
            if let Some(addr) = req.peer_addr() {
                addr.ip().to_string()
            } else {
                "Unknown".to_string()
            }
        );
        message_html.insert_str(
            0,
            "<h3>Please introduce valid data into the form to request an API token",
        );
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!-- src/routes/token/request_token.html -->
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Request sent</title>
    </head>
    <body>
        {message_html}
    </body>
</html>
            "#,
        ))
}
