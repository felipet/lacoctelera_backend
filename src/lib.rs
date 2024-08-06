//! La Coctelera library.

// Re-export of the domain objects.
pub use domain::{IngCategory, Ingredient};

pub mod configuration;
pub mod startup;
pub mod routes {
    mod health;
    pub use health::echo;

    pub mod ingredient {
        mod get;
        mod post;

        pub use get::get_ingredient;
        pub use post::add_ingredient;
    }
}

pub mod domain {
    mod ingredient;

    pub use ingredient::{IngCategory, Ingredient};
}
