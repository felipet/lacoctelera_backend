//! La Coctelera library.

use once_cell::sync::Lazy;
use regex::Regex;
use routes::{health, ingredient::FormData};
use serde::Deserialize;
use utoipa::{
    openapi::{
        security::{ApiKey, ApiKeyValue, SecurityScheme},
        Object, ObjectBuilder,
    },
    IntoParams, Modify, OpenApi, ToSchema,
};
use uuid::Uuid;
use validator::ValidationError;

// Re-export of the domain objects.
pub use domain::{IngCategory, Ingredient};

// Regex to validate an Uuid.
static RE_UUID_V4: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-fA-F0-9-]{4,12}){5}$").unwrap());

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

    pub mod author {
        pub mod auth;
        pub mod delete;
        pub mod get;
        pub mod head;
        pub mod options;
        pub mod patch;

        pub use delete::delete_author;
        pub use get::get_author;
        pub use head::head_author;
        pub use options::options_author;
        pub use patch::patch_author;
    }
}

pub mod domain {
    pub mod author;
    mod error;
    mod ingredient;
    pub mod recipe;
    pub mod tag;

    pub use author::{Author, AuthorBuilder, SocialProfile};
    pub use error::DataDomainError;
    pub use ingredient::{IngCategory, Ingredient};
    pub use recipe::{Recipe, RecipeQuery, StarRate};
    pub use tag::Tag;
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
        routes::author::get::get_author,
        routes::author::patch::patch_author,
        routes::author::delete::delete_author,
        routes::author::options::options_author,
        routes::author::head::head_author,
    ),
    components(
        schemas(Ingredient, IngCategory, FormData, AuthData, health::HealthResponse, health::ServerStatus, domain::Author, domain::SocialProfile, domain::Tag, domain::Recipe, domain::RecipeQuery)
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

/// Custom function to validate a String that should contain an [Uuid].
fn validate_id(value: &Uuid) -> Result<(), ValidationError> {
    if RE_UUID_V4.is_match(&value.to_string()) {
        std::result::Result::Ok(())
    } else {
        Err(ValidationError::new("1"))
    }
}
