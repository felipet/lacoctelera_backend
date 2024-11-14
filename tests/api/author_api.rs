// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::{
    spawn_app, ApiTesterBuilder, Credentials, Resource, TestApp, TestBuilder, TestObject,
};
use actix_web::http::StatusCode;
use lacoctelera::domain::{Author, AuthorBuilder, SocialProfile};
use names::Generator;
use pretty_assertions::assert_eq;
use reqwest::Response;
use sqlx::{Executor, MySqlPool};
use tracing::{debug, error, info};
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

async fn seed_author(pool: &MySqlPool, authors: &[Author]) -> Result<Vec<Uuid>, String> {
    let mut ids = Vec::new();

    let mut transaction = pool.begin().await.expect("Failed to acquire DB");

    for author in authors {
        let id = Uuid::now_v7();
        transaction
            .execute(sqlx::query!(
                "INSERT INTO Author VALUES (?, ?, ?, ?, ?, ?, ?)",
                id.to_string(),
                author.name(),
                author.surname(),
                author.email(),
                author.shareable(),
                author.description(),
                author.website(),
            ))
            .await
            .map_err(|e| {
                error!("{e}");
                "Error seeding authors".to_string()
            })?;
        ids.push(id);

        if let Some(profiles) = author.social_profiles() {
            for profile in profiles {
                transaction
                    .execute(sqlx::query!(
                        r#"
                    INSERT INTO AuthorHashSocialProfile (provider_name, user_name, author_id)
                    VALUES (?, ?, ?);
                    "#,
                        profile.provider_name,
                        profile.website,
                        id.to_string(),
                    ))
                    .await
                    .map_err(|e| {
                        error!("{e}");
                        "Error seeding authors".to_string()
                    })?;
            }
        }
    }

    transaction
        .commit()
        .await
        .expect("Failed to commit authors to the DB");

    Ok(ids)
}

fn valid_author(shareable: bool, social_providers: Option<Vec<SocialProfile>>) -> Author {
    let social_profiles = if let Some(social_providers) = social_providers {
        let mut social_profiles = social_providers.clone();

        social_profiles
            .iter_mut()
            .for_each(|profile| profile.website.insert_str(profile.website.len(), "janedoe"));

        social_profiles
    } else {
        Vec::new()
    };

    let random = Generator::default().next().unwrap();
    let name_and_surname: Vec<&str> = random.split("-").collect();

    AuthorBuilder::default()
        .set_name(name_and_surname[0])
        .set_surname(name_and_surname[1])
        .set_email("janedoe@mail.com")
        .set_shareable(shareable)
        .set_description("A simple description")
        .set_website("https://janedoe.com")
        .set_social_profiles(&social_profiles)
        .build()
        .expect("Failed to build a new Author")
}

/// The DB is preloaded with the supported Social Network providers by the service. So why not loading those for free?
async fn social_network_providers(pool: &MySqlPool) -> Vec<SocialProfile> {
    let record = sqlx::query_as!(
        SocialProfile,
        "SELECT provider_name, website FROM SocialProfile;"
    )
    .fetch_all(pool)
    .await
    .expect("Failed to retrieve Social Network profiles from the test DB");

    record
}

#[actix_web::test]
async fn delete_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (DELETE) -> Attempt to delete a non existing author");
    let id = Uuid::now_v7().to_string();
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    assert_eq!(
        test.delete(&id).await.status().as_u16(),
        StatusCode::BAD_REQUEST
    );

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete an existing author");
    let author = valid_author(false, None);
    debug!("Test author: {:?}", author);

    // Seed the author into the DB.
    let ids = seed_author(test.db_pool(), &[author]).await?;
    let author_id = &ids[0].to_string();

    // Eventually, the error will be Unauthorized. As of today, Actix returns the api_key is missing, thus a
    // bad request.
    assert_eq!(
        test.delete(author_id).await.status().as_u16(),
        StatusCode::BAD_REQUEST
    );

    Ok(())
}

#[actix_web::test]
async fn delete_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (DELETE) -> Attempt to delete a non existing author");
    let id = Uuid::now_v7().to_string();
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    // This might change in the future.
    assert_eq!(test.delete(&id).await.status().as_u16(), StatusCode::OK);

    info!("Test Case::resource::/author (DELETE) -> Attempt to delete an existing author");
    let author = valid_author(false, None);
    debug!("Test author: {:?}", author);

    // Seed the author into the DB.
    let ids = seed_author(test.db_pool(), &[author]).await?;
    let author_id = &ids[0].to_string();

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
    let author_shareable = valid_author(true, None);
    debug!("Test author: {:?}", author_shareable);
    let author_nonshareable = valid_author(false, None);
    debug!("Test author: {:?}", author_nonshareable);
    let ids = seed_author(test.db_pool(), &[author_shareable, author_nonshareable]).await?;

    // Since we are not using an API key, we should not receive private data when the author's profile is private.
    let query = format!("/{}", ids[0]);
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert!(received_author.email().is_some());
    assert!(received_author.description().is_some());
    assert!(received_author.website().is_some());

    let query = format!("/{}", ids[1]);
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    assert!(received_author.email().is_none());
    assert!(received_author.description().is_none());
    assert!(received_author.website().is_some());

    // TODO: Missing social profiles check

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
    let mut author_shareable = valid_author(true, None);
    debug!("Test author: {:?}", author_shareable);
    let mut author_nonshareable = valid_author(false, None);
    debug!("Test author: {:?}", author_nonshareable);
    let ids = seed_author(
        test.db_pool(),
        &[author_shareable.clone(), author_nonshareable.clone()],
    )
    .await?;
    author_shareable.update_from(
        &AuthorBuilder::default()
            .set_id(&ids[0].to_string())
            .build()
            .expect("msg"),
    );

    // Using credentials shall allow us to receive all the attributes of the author's profile.
    let query = format!("/{}", ids[0]);
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    let temporal_author = AuthorBuilder::default()
        .set_id(&received_author.id().unwrap())
        .build()
        .expect("Failed to build a test author");
    author_shareable.update_from(&temporal_author);
    assert_eq!(received_author, author_shareable);

    let query = format!("/{}", ids[1]);
    let response = test.get(&query).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let received_author =
        serde_json::from_str::<Author>(&response.text().await.expect("Failed to parse author"))
            .expect("Failed to deserialize author");
    let temporal_author = AuthorBuilder::default()
        .set_id(&received_author.id().unwrap())
        .build()
        .expect("Failed to build a test author");
    author_nonshareable.update_from(&temporal_author);
    assert_eq!(received_author, author_nonshareable);

    // TODO: Missing social profiles check

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
    let author = valid_author(true, None);
    let ids = seed_author(test.db_pool(), &[author.clone()]).await?;

    assert_eq!(
        test.head(&ids[0].to_string()).await.status().as_u16(),
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

    let author = valid_author(true, None);
    let response = test.post(&author).await;
    // This will change once the backend handles properly unauthorised requests.
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[actix_web::test]
async fn post_with_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (POST) -> Add a new valid author entry");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_with_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let author_base = valid_author(true, None);
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
    assert_eq!(author_base.name(), author.name());
    assert_eq!(author_base.surname(), author.surname());
    assert_eq!(author_base.email(), author.email());
    assert_eq!(author_base.website(), author.website());
    assert_eq!(author_base.description(), author.description());
    assert_eq!(author_base.shareable(), author.shareable());

    Ok(())
}

#[actix_web::test]
async fn patch_no_credentials() -> Result<(), String> {
    info!("Test Case::resource::/author (PATCH) -> Modify an existing author entry");
    let mut test_builder = AuthorApiBuilder::default();
    TestBuilder::author_api_no_credentials(&mut test_builder);
    let test = test_builder.build().await;

    let author = valid_author(true, None);
    let ids = seed_author(test.db_pool(), &[author.clone()]).await?;
    let id = &ids[0].to_string();

    let patched_author = AuthorBuilder::default()
        .set_id(id)
        .set_name("Juana")
        .set_email("juana@mail.com")
        .build()
        .expect("Failed to build an author descriptor");

    let response = test.patch(id, &patched_author).await;

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

    let social_providers = social_network_providers(test.db_pool()).await;

    let author = valid_author(true, Some(social_providers));
    let ids = seed_author(test.db_pool(), &[author.clone()]).await?;
    let id = &ids[0].to_string();

    let patched_author = AuthorBuilder::default()
        .set_id(id)
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

    let response = test.patch(id, &patched_author).await;

    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let response = test.get(&format!("/{id}")).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
    let retrieved_author: Author = serde_json::from_str(
        &response
            .text()
            .await
            .expect("Failed to read response's payload"),
    )
    .expect("Failed to deserialize author");
    assert_eq!(patched_author, retrieved_author);

    Ok(())
}
