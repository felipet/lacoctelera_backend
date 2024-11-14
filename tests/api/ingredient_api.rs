// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::{
    spawn_app, ApiTesterBuilder, Credentials, Resource, TestApp, TestBuilder, TestObject,
};
use actix_web::http::StatusCode;
use lacoctelera::{routes::ingredient::FormData, IngCategory, Ingredient};
use pretty_assertions::assert_eq;
use reqwest::Response;
use sqlx::{Executor, MySqlPool};
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct IngredientApiTester {
    resource: Resource,
    credentials: Credentials,
    test_app: TestApp,
}

#[derive(Default)]
pub struct IngredientApiBuilder {
    credentials: Option<Credentials>,
}

impl ApiTesterBuilder for IngredientApiBuilder {
    type ApiTester = IngredientApiTester;

    fn with_credentials(&mut self) {
        self.credentials = Some(Credentials::WithCredentials);
    }

    fn without_credentials(&mut self) {
        self.credentials = Some(Credentials::NoCredentials);
    }

    async fn build(self) -> IngredientApiTester {
        let credentials = match self.credentials {
            Some(credentials) => credentials,
            None => Credentials::NoCredentials,
        };

        IngredientApiTester::new(credentials).await
    }
}

impl IngredientApiTester {
    pub async fn new(credentials: Credentials) -> Self {
        let mut app = IngredientApiTester {
            resource: Resource::Ingredient,
            credentials,
            test_app: spawn_app().await,
        };

        if credentials == Credentials::WithCredentials {
            app.test_app.generate_access_token().await
        }

        app
    }
}

impl TestObject for IngredientApiTester {
    async fn get(&self, query: &str) -> Response {
        self.test_app
            .get_test(self.resource, self.credentials, query)
            .await
    }

    async fn head(&self, _id: &str) -> Response {
        todo!()
    }

    async fn options(&self) -> Response {
        self.test_app.options_test(self.resource).await
    }

    async fn post<Body: serde::Serialize>(&self, body: &Body) -> Response {
        self.test_app
            .post_test(self.resource, self.credentials, body)
            .await
    }

    async fn delete(&self, _id: &str) -> Response {
        todo!()
    }

    async fn patch<Body: serde::Serialize>(&self, _id: &str, _body: &Body) -> Response {
        todo!()
    }

    fn db_pool(&self) -> &MySqlPool {
        &self.test_app.db_pool
    }
}

type FixtureResult = Result<Vec<Ingredient>, String>;

async fn seed_ingredients(pool: &MySqlPool) -> FixtureResult {
    let test_ingredients = Vec::from([
        Ingredient::parse("Vodka", "spirit", Some("Regular Vodka 40%")).unwrap(),
        Ingredient::parse("White Rum", "spirit", Some("Any white Rum")).unwrap(),
        Ingredient::parse("Lime Super Juice", "other", None).unwrap(),
        Ingredient::parse("Agave Sirup", "other", None).unwrap(),
        Ingredient::parse("Soda water", "soft_drink", None).unwrap(),
        Ingredient::parse(
            "Absolut Vodka",
            "spirit",
            Some("Only Absolut gives the needed flavor profile."),
        )
        .unwrap(),
    ]);

    let mut conn = pool.acquire().await.unwrap();

    for ingredient in test_ingredients.iter() {
        let query = sqlx::query!(
            r#"
            INSERT INTO Ingredient (name, category, `desc`) VALUES
                (?,?,?)
            "#,
            ingredient.name(),
            ingredient.category().to_string(),
            ingredient.desc(),
        );

        conn.execute(query)
            .await
            .expect("Failed to seed ingredients in the DB");
    }

    conn.close().await.map_err(|e| {
        error!("{e}");
        "Failed to seed ingredients into the DB".to_string()
    })?;

    Ok(test_ingredients)
}

#[actix_web::test]
async fn search_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/ingredient (GET) -> Search a non existing ingredient");
    let mut test_builder = IngredientApiBuilder::default();
    TestBuilder::ingredient_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let name = "Vodka";
    let response = test.get(&format!("?name={name}")).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let response_ingredient = serde_json::from_str::<Ingredient>(
        &response
            .text()
            .await
            .expect("Failed to retrieve the payload of the request"),
    );
    assert!(response_ingredient.is_err());

    info!("Test Case::resource::/ingredient (GET) -> Search with wrong format");
    let response = test.get(&format!("?ingredient={name}")).await;
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    info!("Test Case::resource::/ingredient (GET) -> Search an existing ingredient");
    let ingredients = seed_ingredients(test.db_pool()).await?;
    let test_ingredient = &ingredients[0];
    let response = test.get(&format!("?name={}", test_ingredient.name())).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let response_ingredient = serde_json::from_str::<Vec<Ingredient>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve the payload of the request"),
    )
    .expect("Failed to deserialize the response");
    // We pushed two ingredients with the name Vodka into the DB.
    assert!(response_ingredient.len() == 2);
    // The first match should be the closes match.
    assert_eq!(*test_ingredient, response_ingredient[0]);

    Ok(())
}

#[actix_web::test]
async fn search_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/ingredient (GET) -> Search a non existing ingredient");
    let mut test_builder = IngredientApiBuilder::default();
    TestBuilder::ingredient_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let name = "Vodka";
    let response = test.get(&format!("?name={name}")).await;
    // The /ingredient resource does not support credentials as of today.
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[actix_web::test]
async fn post_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/ingredient (POST) -> Add an ingredient using a valid JSON");
    let mut test_builder = IngredientApiBuilder::default();
    TestBuilder::ingredient_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let test_payload = [
        (
            FormData {
                name: "tc1".to_string(),
                category: IngCategory::Spirit.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Spirit test case",
        ),
        (
            FormData {
                name: "tc2".to_string(),
                category: IngCategory::Bitter.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Bitter test case",
        ),
        (
            FormData {
                name: "tc3".to_string(),
                category: IngCategory::Garnish.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Garnish test case",
        ),
        (
            FormData {
                name: "tc4".to_string(),
                category: IngCategory::SoftDrink.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "SoftDrink test case",
        ),
        (
            FormData {
                name: "tc5".to_string(),
                category: IngCategory::Other.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Other test case",
        ),
        (
            FormData {
                name: "My drink 80%".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "Composed name test case",
        ),
        (
            FormData {
                name: "tc7".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "No description teste case",
        ),
    ];

    for (payload, err_msg) in test_payload.iter() {
        debug!("{err_msg}");
        let response = test.post(&payload).await;
        assert_eq!(response.status().as_u16(), 200);
    }

    info!("Test Case::resource::/ingredient (POST) -> Add an ingredient using an invalid JSON");
    let test_payload = [
        (
            FormData {
                name: "1nvalid".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "Wrong name format test case 1",
        ),
        (
            FormData {
                name: "alco;hol".to_string(),
                category: IngCategory::Other.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Wrong name format test case 2",
        ),
        (
            FormData {
                name: "tc3".to_string(),
                category: "my invented category".to_string(),
                desc: None,
            },
            "Non existing category test case",
        ),
    ];

    for (payload, err_msg) in test_payload.iter() {
        debug!("{err_msg}");
        let response = test.post(&payload).await;
        assert_eq!(response.status().as_u16(), 400);
    }

    Ok(())
}
