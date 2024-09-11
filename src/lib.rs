//! La Coctelera library.

use routes::{health, ingredient::FormData};
use serde::Deserialize;
use utoipa::{
    openapi::{
        security::{ApiKey, ApiKeyValue, SecurityScheme},
        Object, ObjectBuilder,
    },
    IntoParams, Modify, OpenApi, ToSchema,
};

// Re-export of the domain objects.
pub use domain::{IngCategory, Ingredient};

pub mod configuration;
pub mod startup;
pub mod telemetry;
pub mod routes {
    pub mod health;
    pub use health::echo;

    pub mod ingredient {
        pub mod get;
        pub mod options;
        pub mod post;

        pub use get::{get_ingredient, QueryData};
        pub use options::options_ingredient;
        pub use post::{add_ingredient, FormData};
    }
}

pub mod domain {
    pub mod author;
    mod ingredient;

    pub use ingredient::{IngCategory, Ingredient};
}

/// Struct that encapsulates valid authentication methods.
///
/// # Description
///
/// Restricted endpoints of the API require the client to include one of the following methods to authenticate:
/// - API key: a token that is shared with clients to allow M2M connections to the API.
#[derive(Deserialize, IntoParams, ToSchema)]
pub struct AuthData {
    /// For token-based authentication methods.
    pub api_key: String,
}

#[allow(dead_code)]
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Query(ApiKeyValue::with_description(
                "api_key",
                "API key token to access restricted endpoints.",
            ))),
        )
    }
}

/// Main [OpenApi] `Struct`. See [the official docs](https://docs.rs/utoipa/latest/utoipa/derive.OpenApi.html).
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::ingredient::get::get_ingredient,
        routes::ingredient::post::add_ingredient,
        routes::ingredient::options::options_ingredient,
        routes::health::echo,
        routes::health::options_echo,
        routes::health::health_check,
        routes::health::options_health,
    ),
    components(
        schemas(Ingredient, IngCategory, FormData, AuthData, health::HealthResponse, health::ServerStatus)
    ),
    tags(
        (name = "Ingredient", description = "Endpoints related to recipe's ingredients."),
        (name = "Maintenance", description = "Endpoints related to server's status.")
    ),
    info(
        title = "La Coctelera API",
        description = "## A REST API for La Coctelera backend service.",
        contact(name = "Felipe Torres González", email = "admin@nubecita.eu")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Generates an [Object] descriptor to reference the custom type [DateTime] in the API docs.
pub fn datetime_object_type() -> Object {
    ObjectBuilder::new()
        .schema_type(utoipa::openapi::SchemaType::String)
        .format(Some(utoipa::openapi::SchemaFormat::Custom(
            "YYYY-MM-DDTHH:MM:SS.NNNNNNNNN+HH:MM".to_string(),
        )))
        .description(Some("A full timestamp with a time offset respect to UTC."))
        .example(Some(serde_json::Value::String(String::from(
            "2025-09-11T08:58:56.121331664+02:00",
        ))))
        .build()
}
