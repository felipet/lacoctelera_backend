//! Module that implements an endopint for health checks.

use actix_web::{get, HttpResponse, Responder};

#[get("/echo")]
pub async fn echo() -> impl Responder {
    HttpResponse::Ok()
        // Avoid caching this endpoint.
        .append_header(("Cache-Control", "no-cache"))
        .finish()
}
