//! La Coctelera library.

use routes::ingredient::FormData;
use utoipa::OpenApi;

// Re-export of the domain objects.
pub use domain::{IngCategory, Ingredient};

pub mod configuration;
pub mod startup;
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
    mod ingredient;

    pub use ingredient::{IngCategory, Ingredient};
}

/// Main [OpenApi] `Struct`. See [the official docs][docs].
/// [docs]: https://docs.rs/utoipa/latest/utoipa/derive.OpenApi.html
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::ingredient::get::get_ingredient,
        routes::ingredient::post::add_ingredient
    ),
    components(
        schemas(Ingredient, IngCategory, FormData)
    ),
    tags(
        (name = "Ingredient", description = "Endpoints related to recipe's ingredients.")
    ),
    info(
        title = "La Coctelera API",
        description = "## A REST API for La Coctelera backend service.",
        contact(name = "Felipe Torres González", email = "admin@nubecita.eu")
    )
)]
pub struct ApiDoc;
