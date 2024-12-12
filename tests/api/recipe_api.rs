// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    fixtures,
    helpers::{
        spawn_app, ApiTesterBuilder, Credentials, Resource, TestApp, TestBuilder, TestObject,
    },
};
use actix_web::http::StatusCode;
use lacoctelera::domain::{QuantityUnit, Recipe, RecipeContains, Tag};
use pretty_assertions::assert_eq;
use reqwest::Response;
use serde::Deserialize;
use sqlx::MySqlPool;
use tracing::info;
use uuid::Uuid;

pub struct RecipeApiTester {
    resource: Resource,
    credentials: Credentials,
    test_app: TestApp,
}

#[derive(Default)]
pub struct RecipeApiBuilder {
    credentials: Option<Credentials>,
}

impl ApiTesterBuilder for RecipeApiBuilder {
    type ApiTester = RecipeApiTester;

    fn with_credentials(&mut self) {
        self.credentials = Some(Credentials::WithCredentials);
    }

    fn without_credentials(&mut self) {
        self.credentials = Some(Credentials::NoCredentials);
    }

    async fn build(self) -> RecipeApiTester {
        let credentials = match self.credentials {
            Some(credentials) => credentials,
            None => Credentials::NoCredentials,
        };

        RecipeApiTester::new(credentials).await
    }
}

impl RecipeApiTester {
    pub async fn new(credentials: Credentials) -> Self {
        let mut app = RecipeApiTester {
            resource: Resource::Recipe,
            credentials,
            test_app: spawn_app().await,
        };

        if credentials == Credentials::WithCredentials {
            app.test_app.generate_access_token().await
        }

        app
    }
}

impl TestObject for RecipeApiTester {
    async fn get(&self, query: &str) -> Response {
        self.test_app
            .get_test(self.resource, self.credentials, query)
            .await
    }

    async fn search(&self, query: &str) -> Response {
        self.test_app
            .search_test(self.resource, self.credentials, query)
            .await
    }

    async fn head(&self, id: &str) -> Response {
        self.test_app.head_test(self.resource, id).await
    }

    async fn options(&self) -> Response {
        self.test_app.options_test(self.resource).await
    }

    async fn post<Body: serde::Serialize>(&self, body: &Body) -> Response {
        self.test_app
            .post_test(self.resource, self.credentials, body)
            .await
    }

    async fn delete(&self, id: &str) -> Response {
        self.test_app
            .delete_test(self.resource, self.credentials, id)
            .await
    }

    async fn patch<Body: serde::Serialize>(&self, id: &str, body: &Body) -> Response {
        self.test_app
            .patch_test(self.resource, self.credentials, id, body)
            .await
    }

    fn db_pool(&self) -> &MySqlPool {
        &self.test_app.db_pool
    }
}

#[actix_web::test]
async fn post_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/recipe (POST) -> Add a new valid recipe entry");
    let mut test_builder = RecipeApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let seed = true;
    let fixture = fixtures::FixtureSeeder::new(test.db_pool())
        .with_ingredients(seed)
        .with_authors(seed)
        .seed()
        .await?;

    let ingredients = fixture
        .ingredient
        .expect("Failed to extract ingredient fixture")
        .valid_fixtures;

    let included_ingredients = &[
        RecipeContains {
            quantity: 1.0,
            unit: QuantityUnit::Ounces,
            ingredient_id: ingredients[0].id().unwrap(),
        },
        RecipeContains {
            quantity: 30.0,
            unit: QuantityUnit::MilliLiter,
            ingredient_id: ingredients[1].id().unwrap(),
        },
    ];

    let authors = fixture
        .author
        .expect("Failed to extract author fixture")
        .valid_fixtures;

    let recipe = Recipe::new(
        None,
        "Dummy Recipe",
        None,
        None,
        None,
        "easy",
        None,
        None,
        included_ingredients,
        &["Pour everything into a cup and enjoy."],
        Some(&authors[0].id().unwrap().to_string()),
    )
    .map_err(|e| e.to_string())?;
    let response = test.post(&recipe).await;
    // This will change once the backend handles properly unauthorised requests.
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[actix_web::test]
async fn post_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/recipe (POST) -> Add a new valid recipe entry");
    let mut test_builder = RecipeApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let seed = true;
    let fixture = fixtures::FixtureSeeder::new(test.db_pool())
        .with_ingredients(seed)
        .with_authors(seed)
        .seed()
        .await
        .expect("Failed to build a fixture");

    let ingredients = fixture
        .ingredient
        .expect("Failed to extract ingredient fixture")
        .valid_fixtures;

    let included_ingredients = &[
        RecipeContains {
            quantity: 1.0,
            unit: QuantityUnit::Ounces,
            ingredient_id: ingredients[0].id().unwrap(),
        },
        RecipeContains {
            quantity: 30.0,
            unit: QuantityUnit::MilliLiter,
            ingredient_id: ingredients[1].id().unwrap(),
        },
    ];

    let authors = fixture
        .author
        .expect("Failed to extract author fixture")
        .valid_fixtures;

    let tags = [
        Tag {
            identifier: "cocktail".to_owned(),
        },
        Tag {
            identifier: "delicious".to_owned(),
        },
    ];

    let author_tags = [
        Tag {
            identifier: "demo".to_owned(),
        },
        Tag {
            identifier: "dummy".to_owned(),
        },
    ];

    let recipe = Recipe::new(
        None,
        "Dummy Recipe",
        None,
        Some(&author_tags),
        Some(&tags),
        "easy",
        Some("A simple description"),
        None,
        included_ingredients,
        &["Pour everything into a cup and enjoy."],
        Some(&authors[0].id().unwrap().to_string()),
    )
    .expect("Failed to build a new recipe");
    let response = test.post(&recipe).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    #[derive(Deserialize)]
    struct Id {
        pub id: Uuid,
    }

    let json = response.json::<Id>().await;
    assert!(json.is_ok());
    let id = json.unwrap();

    let recipe_from_db = sqlx::query!("SELECT * FROM `Cocktail` WHERE `id`=?", id.id.to_string(),)
        .fetch_optional(test.db_pool())
        .await
        .expect("Failed to retrieve a cocktail entry from the DB");

    let recipe_from_db = match recipe_from_db {
        Some(recipe) => recipe,
        None => return Err("Failed to retrieve the inserted recipe from the DB".to_owned()),
    };

    let ingredients_record = sqlx::query!(
        "SELECT * FROM `UsedIngredient` WHERE `cocktail_id`=?",
        id.id.to_string(),
    )
    .fetch_all(test.db_pool())
    .await
    .expect("Failed to retrieve a record from the DB");

    let mut ingredients = Vec::new();

    for record in ingredients_record {
        let split: Vec<&str> = record.amount.split(" ").collect();
        let quantity = split[0]
            .parse::<f32>()
            .expect("Failed to parse the quantity of the ingredient");
        let unit: QuantityUnit = split[1]
            .try_into()
            .expect("Failed to parses the quantity unit");

        ingredients.push(RecipeContains {
            quantity,
            unit,
            ingredient_id: Uuid::parse_str(&record.ingredient_id).expect("Failed to parse UUID"),
        });
    }

    let tags_from_db = sqlx::query!(
        "SELECT * FROM `Tagged` WHERE `cocktail_id`=?",
        id.id.to_string(),
    )
    .fetch_all(test.db_pool())
    .await
    .map_err(|e| e.to_string())?;

    let author_tags: Vec<Tag> = tags_from_db
        .iter()
        .filter(|e| e.r#type == "author")
        .map(|e| Tag {
            identifier: e.tag.clone(),
        })
        .collect();

    let tags: Vec<Tag> = tags_from_db
        .iter()
        .filter(|e| e.r#type == "backend")
        .map(|e| Tag {
            identifier: e.tag.clone(),
        })
        .collect();

    let received_recipe = Recipe::new(
        Some(Uuid::parse_str(&recipe_from_db.id).expect("Failed to parse UUID")),
        &recipe_from_db.name,
        recipe_from_db.image_id.as_deref(),
        Some(&author_tags),
        Some(&tags),
        &recipe_from_db
            .category
            .expect("Failed to extract recipe's category"),
        recipe_from_db.description.as_deref(),
        recipe_from_db.url.as_deref(),
        &ingredients,
        &stepize(&recipe_from_db.steps),
        recipe_from_db.owner.as_deref(),
    )
    .expect("Failed to build a new recipe");

    assert_eq!(recipe.name(), received_recipe.name());
    assert_eq!(recipe.image_id(), received_recipe.image_id());
    assert_eq!(recipe.category(), received_recipe.category());
    assert_eq!(recipe.description(), received_recipe.description());
    assert_eq!(recipe.url(), received_recipe.url());
    assert_eq!(recipe.ingredients(), received_recipe.ingredients());
    assert_eq!(recipe.steps(), received_recipe.steps());
    assert_eq!(recipe.owner(), received_recipe.owner());
    assert_eq!(recipe.tags(), received_recipe.tags());
    assert_eq!(recipe.author_tags(), received_recipe.author_tags());

    Ok(())
}

fn stepize<'a>(steps: &'a str) -> Vec<&'a str> {
    let mut step_list = Vec::new();

    for line in steps.split("/n") {
        step_list.push(line);
    }

    step_list
}
