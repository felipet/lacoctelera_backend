// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::{spawn_app, Credentials, TestApp};
use actix_web::http::StatusCode;
use lacoctelera::domain::{Author, AuthorBuilder, SocialProfile};
use pretty_assertions::assert_eq;
use tracing::{debug, info};
use uuid::Uuid;

async fn valid_author() -> Author {
    let mut social_network_providers = social_network_providers().await;
    social_network_providers
        .iter_mut()
        .for_each(|profile| profile.website.insert_str(profile.website.len(), "janedoe"));

    AuthorBuilder::default()
        .set_name("Jane")
        .set_surname("Doe")
        .set_email("jane_doe@mail.com")
        .set_shareable(true)
        .set_social_profiles(&social_network_providers)
        .build()
        .expect("Failed to build a new Author")
}

/// The DB is preloaded with the supported Social Network providers by the service. So why not loading those for free?
async fn social_network_providers() -> Vec<SocialProfile> {
    let test_app = spawn_app().await;

    let record = sqlx::query_as!(
        SocialProfile,
        "SELECT provider_name, website FROM SocialProfile;"
    )
    .fetch_all(&test_app.db_pool)
    .await
    .expect("Failed to retrive Social Network profiles from the test DB");

    record
}

#[actix_web::test]
async fn post_author_api_with_credentials() {
    let mut test_app = spawn_app().await;
    test_app.generate_access_token().await;
    post_author_with_credentials(&test_app).await;
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn post_author_api_without_credentials() {
    let test_app = spawn_app().await;
    post_author_without_credentials(&test_app).await;
    test_app.db_pool.close().await;
}

async fn post_author_without_credentials(test_app: &TestApp) {
    let result = test_app
        .post_author(&valid_author().await, Credentials::NoCredentials)
        .await;
    let status_code = result.status().as_u16();

    // Check that we get the expected return code.
    assert_eq!(status_code, 400);
}

async fn post_author_with_credentials(test_app: &TestApp) -> Author {
    let result = test_app
        .post_author(&valid_author().await, Credentials::WithCredentials)
        .await;
    let status_code = result.status().as_u16();
    let response_payload = result
        .text()
        .await
        .expect("Failed to read the payload from the POST /author response");

    // Check that we get the expected return code.
    assert_eq!(status_code, 202);
    // Check that the received ID is parseable by Uuid.
    let response_author: Author =
        serde_json::from_str(&response_payload).expect("Failed to parse the response Author");
    debug!("Response author: {:#?}", response_author);

    response_author
}

#[actix_web::test]
async fn get_non_existing_author_api() {
    let test_app = spawn_app().await;
    let author_id = Uuid::now_v7().to_string();
    let response = test_app
        .get_author(&author_id, Credentials::NoCredentials)
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
    let response = test_app
        .get_author(&author_id, Credentials::WithCredentials)
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn insert_and_retrieve_author() {
    let mut test_app = spawn_app().await;
    test_app.generate_access_token().await;
    let test_author = post_author_with_credentials(&test_app).await;
    info!("Insert part passed");
    debug!("New author's ID: {:?}", test_author.id());
    get_author(&test_app, &test_author).await;
    test_app.db_pool.close().await;
}

async fn get_author(test_app: &TestApp, author: &Author) {
    let response = test_app
        .get_author(
            &author.id().expect("No ID for the test Author"),
            Credentials::WithCredentials,
        )
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let retrieved_author: Author = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(author.id(), retrieved_author.id());
    assert_eq!(valid_author().await, retrieved_author);
}

#[actix_web::test]
async fn update_author() {
    let mut test_app = spawn_app().await;
    test_app.generate_access_token().await;

    let test_author = post_author_with_credentials(&test_app).await;
    info!("Insert part passed");
    debug!("New author's ID: {:?}", test_author.id());

    let mut social_providers = social_network_providers().await;

    social_providers
        .iter_mut()
        .for_each(|p| p.website.insert_str(p.website.len(), "juanacierva"));

    let patched_author = AuthorBuilder::default()
        .set_id(test_author.id().as_deref().unwrap())
        .set_name("Juana")
        .set_surname("Cierva")
        .set_email("juana@mail.com")
        .set_description("Una mujer desconocida")
        .set_shareable(false)
        .set_website("https://juana.com")
        .set_social_profiles(&social_providers)
        .build()
        .expect("Failed to build an author descriptor");

    let response = test_app
        .patch_author(&patched_author, Credentials::WithCredentials)
        .await;

    assert_eq!(response.status().as_u16(), StatusCode::ACCEPTED);
    info!("Patch passed");

    let response = test_app
        .get_author(
            &patched_author.id().expect("No ID for the test Author"),
            Credentials::WithCredentials,
        )
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let retrieved_author: Author = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(patched_author, retrieved_author);
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn preflight_request_works() {
    let test_app = spawn_app().await;
    let response = test_app.options_author().await;

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
}

#[actix_web::test]
async fn head_request_works() {
    let mut test_app = spawn_app().await;
    test_app.generate_access_token().await;
    let test_author = post_author_with_credentials(&test_app).await;
    let response = test_app.head_author(&test_author.id().unwrap()).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let response = test_app.head_author(&Uuid::now_v7().to_string()).await;
    assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
}
