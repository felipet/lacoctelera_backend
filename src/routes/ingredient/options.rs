use actix_web::{options, HttpRequest, HttpResponse};

#[options("/ingredient")]
pub async fn options_ingredient(_req: HttpRequest) -> HttpResponse {
    HttpResponse::NoContent()
        .append_header(("access-control-allow-headers", "content-type"))
        .append_header(("access-control-allow-origin", "*"))
        .append_header(("access-control-allow-methods", "GET, POST, OPTIONS"))
        .finish()
}
