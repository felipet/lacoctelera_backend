pub mod configuration;
pub mod startup;
pub mod routes {
    mod health;
    pub use health::echo;

    pub mod ingredient {
        mod get;

        pub use get::get_ingredient;
    }
}

pub mod domain {
    mod ingredient;

    pub use ingredient::{IngCategory, Ingredient};
}
