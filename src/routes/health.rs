//! Module that implements an endopint for health checks.
//!
//! # Description
//!
//! Two endpoints are available:
//! - [echo] for a basic ping support with public access.
//! - [health_check] for a detailed health report with restricted access.
//!
//! The number of requests within a time frame to both endpoints are limited by the API to every client. This is
//! a mechanism to prevent DoS attacks to the server. Every response includes the header *Retry-After* to inform the
//! client when it is allowed to send a new request to the API.

use crate::{datetime_object_type, AuthData};
use actix_web::{get, options, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Days, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::instrument;
use utoipa::{
    openapi::{
        example::ExampleBuilder,
        schema::{Object, ObjectBuilder},
        ContentBuilder, Header, RefOr, Response, ResponseBuilder, ResponsesBuilder, SchemaType,
    },
    IntoResponses, ToSchema,
};

/// Enum that identifies the status of the server.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum ServerStatus {
    /// The server is running smoothly.
    Ok,
    /// The server is overloaded. Expect longer service times.
    Overloaded,
    /// Scheduled maintenance.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    MaintenanceScheduled(DateTime<Local>),
    /// Server under maintenance. The given timestamp forecasts when the server is expected to be online again.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    OnMaintenance(DateTime<Local>),
    /// The connection with the DB server is lost.
    DbDown,
    /// The server is not able to accept new requests.
    Down,
    /// API token expired. Proceed to renew the token to continue using the restricted endpoints.
    TokenExpired,
}

/// Struct that holds status information of the running instance of the application.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Current server status, see [ServerStatus].
    pub server_status: ServerStatus,
    /// Expire date of the used API token.
    #[schema(schema_with = datetime_object_type)]
    pub api_expire_time: DateTime<Local>,
}

impl HealthResponse {
    /// A simple example of the struct's fields when the server is running Ok.
    pub fn example_ok() -> HealthResponse {
        HealthResponse {
            server_status: ServerStatus::Ok,
            api_expire_time: Local::now().checked_add_days(Days::new(1)).unwrap(),
        }
    }

    /// A simple example of the struct's fields when the server has a scheduled maintenance.
    pub fn example_maintenance_scheduled() -> HealthResponse {
        let ts = Local::now().checked_add_days(Days::new(1)).unwrap();
        HealthResponse {
            server_status: ServerStatus::MaintenanceScheduled(ts.clone()),
            api_expire_time: ts,
        }
    }
}

impl IntoResponses for HealthResponse {
    fn responses() -> BTreeMap<String, RefOr<Response>> {
        let mut cache_control_header = Header::new(Object::with_type(SchemaType::String));
        cache_control_header.description = Some(String::from(
            "Set to *no-cache* to avoid caching maintenance information.",
        ));

        let mut retry_after_header = Header::new(
            ObjectBuilder::new()
                .default(Some(serde_json::Value::String(String::from("algo"))))
                .schema_type(SchemaType::String)
                .build(),
        );
        retry_after_header.description = Some(String::from(
            "How many seconds the client shall wait before issuing a new request.",
        ));

        ResponsesBuilder::new()
            .response(
                "200",
                ResponseBuilder::new()
                    .description("**Ok**")
                    .header("Cache-Control", cache_control_header.clone())
                    .header("Retry-After", retry_after_header.clone())
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(HealthResponse::schema().1)
                            .examples_from_iter(BTreeMap::from([
                                (
                                    "Ok example",
                                    ExampleBuilder::new()
                                        .summary("An example response of the server running smoothly.")
                                        .value(Some(
                                            serde_json::to_value(HealthResponse::example_ok()).unwrap(),
                                        ))
                                        .build(),
                                ),
                                (
                                    "Scheduled maintenance example",
                                    ExampleBuilder::new()
                                    .summary("An example response of a scheduled maintenance of the server.")
                                    .value(Some(
                                        serde_json::to_value(HealthResponse::example_maintenance_scheduled()).unwrap(),
                                    ))
                                    .build(),
                                )
                            ]))
                            .build(),
                    ),
            )
            .response(
                "429",
                ResponseBuilder::default()
                    .description("**Too many requests.**")
                    .header("Cache-Control", cache_control_header.clone())
                    .header("Retry-After", retry_after_header.clone()),
            )
            .response("401",
                ResponseBuilder::default()
                .description("**Unauthorised access to a restricted endpoint.**")
                .header("Cache-Control", cache_control_header)
                .header("Retry-After", retry_after_header),
            )
            .build()
            .into()
    }
}

/// Ping endpoint for the API (Public).
///
/// # Description
///
/// This public endpoint shall be used by clients of the API to check whether the server is alive and ready to accept
/// new requests or not.
///
/// The number of allowed requests by a single client is limited to 1 per minute. If this value is reached by a client,
/// the client is banned for an amount of time, which is specified by the header *Retry-After*. The ban time increases
/// exponentially when a client reaches the threshold multiple times.
#[utoipa::path(
    get,
    tag = "Maintenance",
    responses(
        (
            status = 200, description = "**Ok**",
            headers(
                ("Cache-Control", description = "Cache control is set to *no-cache*."),
                ("Retry-After", description = "Amount of time between requests (seconds).")
            )
        ),
        (
            status = 429, description = "**Too many requests.**",
            headers(
                ("Cache-Control", description = "Cache control is set to *no-cache*."),
                ("Retry-After", description = "Amount of time between requests (seconds).")
            )
        )
    )
)]
#[instrument()]
#[get("/echo")]
pub async fn echo() -> impl Responder {
    HttpResponse::NotImplemented()
        // Avoid caching this endpoint.
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Retry-After", "60"))
        .finish()
}

/// Options method for the /echo endpoint.
#[utoipa::path(
    options,
    tag = "Maintenance",
    responses(
        (
            status = 204,
            description = "Supported requests to the /echo endpoint",
            headers(
                ("access-control-allow-origin", description = "*"),
                ("access-control-allow-methods", description = "GET, OPTIONS"),
                ("cache-control", description = "public, max-age=604800")
            )
        )
    )
)]
#[options("/echo")]
pub async fn options_echo() -> impl Responder {
    HttpResponse::NotImplemented()
        .append_header(("access-control-allow-origin", "*"))
        .append_header(("cache-control", "public, max-age=604800"))
        .append_header(("access-control-allow-methods", "GET, OPTIONS"))
        .finish()
}

/// Health status endpoint for the API (Restricted).
///
/// # Description
///
/// This restricted endpoint allows authorized clients to retrieve a health report of the server.
///
/// The number of allowed requests by a single client is limited to 2 per minute. If this value is reached by a client,
/// the client is banned for an amount of time, which is specified by the header *Retry-After*. The ban time increases
/// exponentially when a client reaches the threshold multiple times.
#[utoipa::path(
    get,
    tag = "Maintenance",
    responses(HealthResponse),
    security(
        ("api_key" = [])
    ),
)]
#[instrument(skip(req))]
#[get("/health")]
pub async fn health_check(req: web::Query<AuthData>) -> impl Responder {
    if req.api_key != "" {
        HttpResponse::NotImplemented()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .append_header(("access-control-allow-headers", "content-type"))
            // Avoid caching this endpoint.
            .append_header(("Cache-Control", "no-cache"))
            .append_header(("Retry-After", "60"))
            .finish()
    } else {
        HttpResponse::Unauthorized()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .append_header(("access-control-allow-headers", "content-type"))
            // Avoid caching this endpoint.
            .append_header(("Cache-Control", "no-cache"))
            .append_header(("Retry-After", "60"))
            .finish()
    }
}

/// Options method for the /health endpoint.
#[utoipa::path(
    options,
    path = "/health",
    tag = "Maintenance",
    responses(
        (
            status = 204,
            description = "Supported requests to the /health endpoint",
            headers(
                ("access-control-allow-origin", description = "*"),
                ("access-control-allow-methods", description = "GET, OPTIONS"),
                ("cache-control", description = "public, max-age=604800")
            )
        )
    )
)]
#[instrument(skip(_req))]
#[options("/health")]
pub async fn options_health(_req: HttpRequest) -> HttpResponse {
    HttpResponse::NoContent()
        .append_header(("access-control-allow-origin", "*"))
        .append_header(("access-control-allow-methods", "GET, OPTIONS"))
        .append_header(("cache-control", "public, max-age=604800"))
        .finish()
}
