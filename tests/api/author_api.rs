// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    fixtures::{self, AuthorFixture},
    helpers::{
        spawn_app, ApiTesterBuilder, Credentials, Resource, TestApp, TestBuilder, TestObject,
    },
};
use actix_web::http::StatusCode;
use lacoctelera::domain::{Author, AuthorBuilder, SocialProfile};
use pretty_assertions::assert_eq;
use reqwest::Response;
use sqlx::MySqlPool;
use std::iter::zip;
use tracing::info;
use uuid::Uuid;

pub struct AuthorApiTester {
    resource: Resource,
    credentials: Credentials,
    test_app: TestApp,
}

#[derive(Default)]
pub struct AuthorApiBuilder {
    credentials: Option<Credentials>,
}

impl ApiTesterBuilder for AuthorApiBuilder {
    type ApiTester = AuthorApiTester;

    fn with_credentials(&mut self) {
        self.credentials = Some(Credentials::WithCredentials);
    }

    fn without_credentials(&mut self) {
        self.credentials = Some(Credentials::NoCredentials);
    }

    async fn build(self) -> AuthorApiTester {
        let credentials = match self.credentials {
            Some(credentials) => credentials,
            None => Credentials::NoCredentials,
        };

        AuthorApiTester::new(credentials).await
    }
}

impl AuthorApiTester {
    pub async fn new(credentials: Credentials) -> Self {
        let mut app = AuthorApiTester {
            resource: Resource::Author,
            credentials,
            test_app: spawn_app().await,
        };

        if credentials == Credentials::WithCredentials {
            app.test_app.generate_access_token().await
        }

        app
    }
}

impl TestObject for AuthorApiTester {
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
async fn delete_no_credentials() -> Result<(), String> {
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete a non existing author");
    let id = Uuid::now_v7().to_string();

    assert_eq!(
        test.delete(&id).await.status().as_u16(),
        StatusCode::BAD_REQUEST
    );

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete an existing author");
    // Seed the author into the DB.
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    let without_social_media = false;
    author_fixture
        .seed(test.db_pool(), without_social_media)
        .await?;

    let author_id = author_fixture.valid_fixtures[0]
        .id()
        .expect("Failed to unwrap fixture author's ID")
        .to_string();

    // Eventually, the error will be Unauthorized. As of today, Actix returns the api_key is missing, thus a
    // bad request.
    assert_eq!(
        test.delete(&author_id).await.status().as_u16(),
        StatusCode::BAD_REQUEST
    );

    Ok(())
}

#[actix_web::test]
async fn delete_with_credentials() -> Result<(), String> {
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete using a wrong author ID");
    let id = rand::random::<i32>().to_string();
    let response = test.delete(&id).await;
    assert_eq!(
        response.status().as_u16(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    info!("Test Case::resource::/author (DELETE) -> Attempt to delete a non existing author");
    let id = Uuid::now_v7().to_string();

    // This might change in the future.
    assert_eq!(test.delete(&id).await.status().as_u16(), StatusCode::OK);

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete an existing author");
    let mut social_profile_fixture = fixtures::SocialProfileFixture::default();
    social_profile_fixture.load()?;
    social_profile_fixture.seed(test.db_pool()).await?;

    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    let with_social_media = true;
    author_fixture
        .seed(test.db_pool(), with_social_media)
        .await?;

    let author_id = &author_fixture.valid_fixtures[0]
        .id()
        .expect("Failed to unwrap fixture author's ID")
        .to_string();

    // Eventually, the error will be Unauthorized. As of today, Actix returns the api_key is missing, thus a
    // bad request.
    assert_eq!(
        test.delete(author_id).await.status().as_u16(),
        StatusCode::OK
    );

    Ok(())
}

#[actix_web::test]
async fn get_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (GET) -> Request an author whose ID doesn't exist");
    let author_id = Uuid::now_v7().to_string();
    let query = format!("/{author_id}");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;
    assert_eq!(
        test.get(&query).await.status().as_u16(),
        StatusCode::NOT_FOUND
    );

    info!("Test Case::resource::/author (GET) -> Request an author whose ID does exist");
    // Let's get two author instances.
    let mut social_profile_fixture = fixtures::SocialProfileFixture::default();
    social_profile_fixture.load()?;
    social_profile_fixture.seed(test.db_pool()).await?;

    let with_social_media = true;
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture
        .seed(test.db_pool(), with_social_media)
        .await?;

    let author_shareable = &author_fixture.valid_fixtures[0];
    let author_nonshareable = &author_fixture.valid_fixtures[1];

    // Since we are not using an API key, we should not receive private data when the author's profile is private.
    let query = format!(
        "/{}",
        author_fixture.valid_fixtures[0]
            .id()
            .expect("Failed to unwrap fixture author's ID")
    );
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert!(received_author.email().is_some());
    assert!(received_author.description().is_some());
    assert!(received_author.website().is_some());

    for (p1, p2) in zip(
        received_author.social_profiles().unwrap(),
        author_shareable.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    let query = format!(
        "/{}",
        author_fixture.valid_fixtures[1]
            .id()
            .expect("Failed to unwrap fixture author's ID")
    );
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert!(received_author.email().is_none());
    assert!(received_author.description().is_none());
    assert!(received_author.website().is_some());

    for (p1, p2) in zip(
        received_author.social_profiles().unwrap(),
        author_nonshareable.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    Ok(())
}

#[actix_web::test]
async fn get_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (GET) -> Request an author whose ID doesn't exist");
    let author_id = Uuid::now_v7().to_string();
    let query = format!("/{author_id}");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;
    assert_eq!(
        test.get(&query).await.status().as_u16(),
        StatusCode::NOT_FOUND
    );

    info!("Test Case::resource::/author (GET) -> Request an author whose ID does exist");
    // Let's get two author instances.
    let mut social_profile_fixture = fixtures::SocialProfileFixture::default();
    social_profile_fixture.load()?;
    social_profile_fixture.seed(test.db_pool()).await?;

    let with_social_media = true;
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture
        .seed(test.db_pool(), with_social_media)
        .await?;

    let author_shareable = &author_fixture.valid_fixtures[0];
    let author_nonshareable = &author_fixture.valid_fixtures[1];

    // Using credentials shall allow us to receive all the attributes of the author's profile.
    let query = format!(
        "/{}",
        author_shareable
            .id()
            .expect("Failed to unwrap fixture author's ID")
    );
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert_eq!(received_author.id(), author_shareable.id());
    assert_eq!(received_author.name(), author_shareable.name());
    assert_eq!(received_author.surname(), author_shareable.surname());
    assert_eq!(
        received_author.description(),
        author_shareable.description()
    );
    assert_eq!(received_author.website(), author_shareable.website());
    assert_eq!(received_author.email(), author_shareable.email());
    assert_eq!(received_author.shareable(), author_shareable.shareable());
    for (p1, p2) in zip(
        received_author.social_profiles().unwrap(),
        author_shareable.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    let query = format!(
        "/{}",
        author_nonshareable
            .id()
            .expect("Failed to unwrap fixture author's ID")
    );
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert_eq!(received_author.id(), author_nonshareable.id());
    assert_eq!(received_author.name(), author_nonshareable.name());
    assert_eq!(received_author.surname(), author_nonshareable.surname());
    assert_eq!(
        received_author.description(),
        author_nonshareable.description()
    );
    assert_eq!(received_author.website(), author_nonshareable.website());
    assert_eq!(received_author.email(), author_nonshareable.email());
    assert_eq!(received_author.shareable(), author_nonshareable.shareable());

    for (p1, p2) in zip(
        received_author.social_profiles().unwrap(),
        author_nonshareable.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    Ok(())
}

#[actix_web::test]
async fn head() -> Result<(), String> {
    info!("Test Case::resource::/author (HEAD) -> Attempt to request a non existing client");
    let id = Uuid::now_v7().to_string();
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    assert_eq!(
        test.head(&id).await.status().as_u16(),
        StatusCode::NOT_FOUND
    );

    info!("Test Case::resource::/author (HEAD) -> Attempt to request an existing client");
    let with_social_media = false;
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture
        .seed(test.db_pool(), with_social_media)
        .await?;
    let author_shareable = &author_fixture.valid_fixtures[0];

    assert_eq!(
        test.head(
            &author_shareable
                .id()
                .expect("Failed to extract ID")
                .to_string()
        )
        .await
        .status()
        .as_u16(),
        StatusCode::OK
    );

    Ok(())
}

#[actix_web::test]
async fn options() -> Result<(), String> {
    info!("Test Case::resource::/author (OPTIONS) -> Preflight check");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;
    let response = test.options().await;

    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(
        headers.get("access-control-allow-headers").unwrap(),
        &"content-type"
    );

    let headers = headers
        .get("access-control-allow-methods")
        .unwrap()
        .to_str()
        .expect("Failed to parse headers");
    let allowed_methods = &["GET", "POST", "PATCH", "DELETE", "HEAD"];
    for method in allowed_methods {
        assert!(headers.contains(method));
    }

    Ok(())
}

#[actix_web::test]
async fn post_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (POST) -> Add a new valid author entry");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let with_social_media = false;
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture
        .seed(test.db_pool(), with_social_media)
        .await?;
    let author = &author_fixture.valid_fixtures[0];
    let response = test.post(author).await;
    // This will change once the backend handles properly unauthorised requests.
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[actix_web::test]
async fn post_with_credentials() -> Result<(), String> {
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    info!("Test Case::resource::/author (POST) -> Add a new valid author entry");
    let mut social_profile_fixture = fixtures::SocialProfileFixture::default();
    social_profile_fixture.load()?;
    social_profile_fixture.seed(test.db_pool()).await?;

    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;

    let author_template = &author_fixture.valid_fixtures[0];

    let mut author_profiles = Vec::new();

    for profile in social_profile_fixture.valid_fixtures {
        author_profiles.push(SocialProfile {
            provider_name: profile.provider_name,
            website: author_template.name().unwrap().to_owned(),
        });
    }

    let author_base = AuthorBuilder::default()
        .set_name(author_template.name().unwrap())
        .set_surname(author_template.surname().unwrap())
        .set_email(author_template.email().unwrap())
        .set_description(author_template.description().unwrap())
        .set_shareable(author_template.shareable())
        .set_website(author_template.website().unwrap())
        .set_social_profiles(&author_profiles)
        .build()
        .map_err(|e| format!("Failed to build a test author using a builder: {e}"))?;

    let response = test.post(&author_base).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Author>(
        &response
            .text()
            .await
            .expect("Failed to extract the payload"),
    )
    .expect("Failed to deserialize payload");
    let received_author = serde_json::from_str::<Author>(
        &test
            .get(&format!(
                "/{}",
                payload.id().expect("Failed to extract ID").to_string()
            ))
            .await
            .text()
            .await
            .unwrap(),
    )
    .expect("Failed to parse the received author");
    assert_eq!(received_author.name(), author_base.name());
    assert_eq!(received_author.surname(), author_base.surname());
    assert_eq!(received_author.description(), author_base.description());
    assert_eq!(received_author.website(), author_base.website());
    assert_eq!(received_author.email(), author_base.email());
    assert_eq!(received_author.shareable(), author_base.shareable());
    for (p1, p2) in zip(
        received_author.social_profiles().unwrap(),
        author_base.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    info!(
        "Test Case::resource::/author (POST) -> Add a new valid author entry using default values"
    );
    let author_base = AuthorBuilder::default()
        .set_email("demo@mail.com")
        .build()
        .expect("Failed to build test author");
    let response = test.post(&author_base).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Author>(
        &response
            .text()
            .await
            .expect("Failed to extract the payload"),
    )
    .expect("Failed to deserialize payload");
    let author = serde_json::from_str::<Author>(
        &test
            .get(&format!(
                "/{}",
                payload.id().expect("Failed to extract ID").to_string()
            ))
            .await
            .text()
            .await
            .unwrap(),
    )
    .expect("Failed to parse the received author");
    // Author's name and surname shall be given random values from the backend.
    assert!(author.name().is_some());
    assert!(author.surname().is_some());
    assert_eq!(author_base.email(), author.email());
    assert_eq!(author_base.shareable(), author.shareable());

    Ok(())
}

#[actix_web::test]
async fn patch_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (PATCH) -> Modify an existing author entry");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture.seed(test.db_pool(), false).await?;
    let author = &author_fixture.valid_fixtures[0];

    let patched_author = AuthorBuilder::default()
        .set_id(&author.id().unwrap())
        .set_name("Juana")
        .set_email("juana@mail.com")
        .build()
        .expect("Failed to build an author descriptor");

    let response = test.patch(&author.id().unwrap(), &patched_author).await;

    // This will change once the backend implements a proper unauthorised response.
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[actix_web::test]
async fn patch_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (PATCH) -> Modify an existing author entry");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let mut social_profile_fixture = fixtures::SocialProfileFixture::default();
    social_profile_fixture.load()?;
    social_profile_fixture.seed(test.db_pool()).await?;

    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture.seed(test.db_pool(), true).await?;

    let author_template = &author_fixture.valid_fixtures[0];

    let mut author_profiles = Vec::new();

    for profile in social_profile_fixture.valid_fixtures {
        author_profiles.push(SocialProfile {
            provider_name: profile.provider_name,
            website: author_template.name().unwrap().to_owned(),
        });
    }

    let author = AuthorBuilder::default()
        .set_id(&author_template.id().unwrap())
        .set_name(author_template.name().unwrap())
        .set_surname(author_template.surname().unwrap())
        .set_email(author_template.email().unwrap())
        .set_description(author_template.description().unwrap())
        .set_shareable(author_template.shareable())
        .set_website(author_template.website().unwrap())
        .set_social_profiles(&author_profiles)
        .build()
        .map_err(|e| format!("Failed to build a test author using a builder: {e}"))?;

    let patched_author = AuthorBuilder::default()
        .set_id(&author.id().unwrap())
        .set_name("Juana")
        .set_surname("Cierva")
        .set_email("juana@mail.com")
        .set_description("Una mujer desconocida")
        .set_shareable(true)
        .set_website("https://juana.com")
        .set_social_profiles(
            author
                .social_profiles()
                .expect("Failed to pass social profiles"),
        )
        .build()
        .expect("Failed to build an author descriptor");

    let response = test.patch(&author.id().unwrap(), &patched_author).await;

    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let response = test.get(&format!("/{}", &author.id().unwrap())).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let retrieved_author: Author = serde_json::from_str(
        &response
            .text()
            .await
            .expect("Failed to read response's payload"),
    )
    .expect("Failed to deserialize author");

    assert_eq!(retrieved_author.name(), patched_author.name());
    assert_eq!(retrieved_author.surname(), patched_author.surname());
    assert_eq!(retrieved_author.description(), patched_author.description());
    assert_eq!(retrieved_author.website(), patched_author.website());
    assert_eq!(retrieved_author.email(), patched_author.email());
    assert_eq!(retrieved_author.shareable(), patched_author.shareable());
    for (p1, p2) in zip(
        retrieved_author.social_profiles().unwrap(),
        patched_author.social_profiles().unwrap(),
    ) {
        if !p1.website.contains(&p2.website) {
            println!("{:?}", p2);
            println!("{:?}", p1);
            panic!("Social profile not found");
        }
    }

    Ok(())
}

#[actix_web::test]
async fn search_no_credentials() -> Result<(), String> {
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    info!("Test Case::resource::/author (GET) -> Search a nonexisting author");
    let search = AuthorBuilder::default()
        .set_name("Jane")
        .build()
        .expect("Failed to build a test author");
    let query = format!("?name={}", search.name().unwrap());
    let response = test.search(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert!(payload.len() == 0);

    info!("Test Case::resource::/author (GET) -> Search existing authors");
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture.seed(test.db_pool(), false).await?;
    let author_shareable = &author_fixture.valid_fixtures[0];
    let author_no_shareable = &author_fixture.valid_fixtures[1];

    let query = format!("?name={}", author_shareable.name().unwrap());
    let response = test.search(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert_eq!(payload.len(), 1);
    let response_author = &payload[0];
    assert_eq!(author_shareable.id(), response_author.id());
    assert_eq!(author_shareable.name(), response_author.name());
    assert_eq!(author_shareable.surname(), response_author.surname());
    assert_eq!(author_shareable.email(), response_author.email());
    assert_eq!(
        author_shareable.description(),
        response_author.description()
    );
    assert_eq!(author_shareable.website(), response_author.website());

    let query = format!("?name={}", author_no_shareable.name().unwrap());
    let response = test.search(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert_eq!(payload.len(), 1);
    let response_author = &payload[0];
    assert_eq!(author_no_shareable.id(), response_author.id());
    assert_eq!(author_no_shareable.name(), response_author.name());
    assert_eq!(author_no_shareable.surname(), response_author.surname());
    assert_eq!(None, response_author.email());
    assert_eq!(None, response_author.description());
    assert_eq!(author_no_shareable.website(), response_author.website());

    Ok(())
}

#[actix_web::test]
async fn search_with_credentials() -> Result<(), String> {
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    info!("Test Case::resource::/author (GET) -> Search a nonexisting author");
    let search = AuthorBuilder::default()
        .set_name("Jane")
        .build()
        .expect("Failed to build a test author");
    let query = format!("?name={}", search.name().unwrap());
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert!(payload.len() == 0);

    info!("Test Case::resource::/author (GET) -> Search existing authors");
    let mut author_fixture = AuthorFixture::default();
    author_fixture.load()?;
    author_fixture.seed(test.db_pool(), false).await?;
    let author_shareable = &author_fixture.valid_fixtures[0];
    let author_no_shareable = &author_fixture.valid_fixtures[1];

    let query = format!("?name={}", author_shareable.name().unwrap());
    let response = test.search(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert_eq!(payload.len(), 1);
    let response_author = &payload[0];
    assert_eq!(author_shareable.id(), response_author.id());
    assert_eq!(author_shareable.name(), response_author.name());
    assert_eq!(author_shareable.surname(), response_author.surname());
    assert_eq!(author_shareable.email(), response_author.email());
    assert_eq!(
        author_shareable.description(),
        response_author.description()
    );
    assert_eq!(author_shareable.website(), response_author.website());

    let query = format!("?name={}", author_no_shareable.name().unwrap());
    let response = test.search(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let payload = serde_json::from_str::<Vec<Author>>(
        &response
            .text()
            .await
            .expect("Failed to retrieve response's payload"),
    )
    .expect("Failed to deserialize the payload");
    assert_eq!(payload.len(), 1);
    let response_author = &payload[0];
    assert_eq!(author_no_shareable.id(), response_author.id());
    assert_eq!(author_no_shareable.name(), response_author.name());
    assert_eq!(author_no_shareable.surname(), response_author.surname());
    assert_eq!(author_no_shareable.email(), response_author.email());
    assert_eq!(
        author_no_shareable.description(),
        response_author.description()
    );
    assert_eq!(author_no_shareable.website(), response_author.website());

    Ok(())
}
