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
use lacoctelera::domain::{QuantityUnit, Recipe, RecipeContains};
use pretty_assertions::assert_eq;
use reqwest::Response;
use sqlx::{Executor, MySqlPool};
use tracing::{debug, error, info};
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

// #[actix_web::test]
// async fn post_no_credentials() -> Result<(), String> {
//     info!("Test Case::resource::/recipe (POST) -> Add a new valid recipe entry");
//     let mut test_builder = RecipeApiBuilder::default();
//     TestBuilder::author_api_no_credentials(&mut test_builder);
//     let test = test_builder.build().await;

//     let ingredients = [RecipeContains {
//         quantity: 1.0,
//         unit: QuantityUnit::Cups,
//         ingredient_id: Uuid::now_v7(),
//     }];

//     let recipe = Recipe::new(
//         None,
//         "Dummy Recipe",
//         None,
//         None,
//         None,
//         "easy",
//         None,
//         None,
//         &ingredients,
//         &["Pour everything into a cup and enjoy."],
//         Some(&Uuid::now_v7().to_string()),
//     )
//     .map_err(|e| e.to_string())?;
//     let response = test.post(&recipe).await;
//     // This will change once the backend handles properly unauthorised requests.
//     assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

//     Ok(())
// }

// #[actix_web::test]
// async fn post_with_credentials() -> Result<(), String> {
//     info!("Test Case::resource::/recipe (POST) -> Add a new valid recipe entry");
//     let mut test_builder = RecipeApiBuilder::default();
//     TestBuilder::author_api_with_credentials(&mut test_builder);
//     let test = test_builder.build().await;

//     let author = valid_author(true, None);
//     let author_id = seed_author(test.db_pool(), &[author]).await?;

//     let ingredients = [RecipeContains {
//         quantity: 1.0,
//         unit: QuantityUnit::Cups,
//         ingredient_id: Uuid::now_v7(),
//     }];

//     let recipe = Recipe::new(
//         None,
//         "Dummy Recipe",
//         None,
//         None,
//         None,
//         "easy",
//         None,
//         None,
//         &ingredients,
//         &["Pour everything into a cup and enjoy."],
//         None,
//     )
//     .map_err(|e| e.to_string())?;
//     let response = test.post(&recipe).await;
//     assert_eq!(response.status().as_u16(), StatusCode::OK);
//     assert!(Uuid::parse_str(&response.text().await.expect("Failed to retrieve response")).is_ok());

//     Ok(())
// }
