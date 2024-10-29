use crate::helpers::spawn_app;
use chrono::{Local, TimeDelta};
use lacoctelera::{
    authentication::*,
    domain::{ClientId, DataDomainError},
};
use pretty_assertions::assert_eq;
use secrecy::SecretString;
use sqlx::{Executor, MySqlPool};
use tracing::{error, info};

async fn seed_api_client(pool: &MySqlPool) -> Result<ClientId, anyhow::Error> {
    let client_id = ClientId::new();
    let query = sqlx::query!(
        r#"
        INSERT INTO ApiUser (id, name, email, validated, enabled, explanation)
        VALUES (?, ?, ?, 0, 0, ?);
        "#,
        client_id.to_string(),
        "Test Client",
        "test_client@mail.com",
        "No explanation",
    );

    pool.execute(query).await?;

    Ok(client_id)
}

#[actix_web::test]
async fn token_request_returns_202_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = serde_json::json!({
        "email": "janedoe@mail.com",
        "explanation": "A_very_long_sentence_for_testing",
    });

    let response = test_app.post_token_request(&body).await;

    assert_eq!(202, response.status().as_u16());

    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn token_request_returns_200_for_existing_email() {
    let test_app = spawn_app().await;

    let body = serde_json::json!({
        "email": "janedoe@mail.com",
        "explanation": "A_very_long_sentence_for_testing",
    });

    let response = test_app.post_token_request(&body).await;

    // The first time, it shall return Ok (202).
    assert_eq!(202, response.status().as_u16());

    // Attempt to register twice the same email.
    let response = test_app.post_token_request(&body).await;

    // This time, the response shall be 406, as the email is already used.
    assert_eq!(406, response.status().as_u16());

    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn register_new_token_request() {
    let test_app = spawn_app().await;

    let body = serde_json::json!({
        "email": "janedoe@mail.com",
        "explanation": "A_very_long_sentence_for_testing",
    });
    let email = "janedoe@mail.com";

    let response = test_app.post_token_request(&body).await;

    assert_eq!(202, response.status().as_u16());

    let record = sqlx::query!(
        "SELECT id, validated, enabled FROM ApiUser WHERE email = ?",
        email
    )
    .fetch_optional(&test_app.db_pool)
    .await
    .expect("Failed to search test user data in the DB")
    .unwrap();

    assert_eq!(record.enabled, Some(0));
    assert_eq!(record.validated, Some(0));
    let client_id = record.id;

    let record = sqlx::query!(
        "SELECT api_token, valid_until FROM ApiToken WHERE client_id = ?",
        client_id
    )
    .fetch_optional(&test_app.db_pool)
    .await
    .expect("Failed to search test user data in the DB")
    .unwrap();

    assert!(record.api_token.len() == 25);
    assert_eq!(
        record.valid_until.date_naive(),
        Local::now().date_naive() + TimeDelta::days(1)
    );

    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}

/// Test to check all the utility functions that handle tokens and the DB.
#[actix_web::test]
async fn token_management() {
    let test_app = spawn_app().await;

    // First: seed an ApiClient into the DB. Tokens need a valid reference to a client.
    let client_id = seed_api_client(&test_app.db_pool)
        .await
        .expect("Failed to seed an ApiClient into the DB");

    // Now, we can handle tokens in the DB.
    let plain_token = generate_token();
    let token_hashed = generate_new_token_hash(SecretString::from(plain_token))
        .expect("Failed to generate the token hash");
    let mut transaction = test_app
        .db_pool
        .begin()
        .await
        .expect("Failed to begin a new DB transaction");
    store_validation_token(
        &mut transaction,
        &token_hashed,
        TimeDelta::days(1),
        &client_id,
    )
    .await
    .expect("Failed to store the token in the DB");
    transaction
        .commit()
        .await
        .expect("Failed to commit transaction to the DB");

    // At this point, the API client should have access to the API. However, that is checked in another test case,
    // let's simply wipe that token and call it a day.
    delete_token(&test_app.db_pool, token_hashed)
        .await
        .expect("Failed to delete the token from the DB");

    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn non_existing_client_has_no_access_to_the_api() {
    let test_app = spawn_app().await;
    let non_existing_client = ClientId::new();
    info!("Non existing ClientId = {non_existing_client}");
    let token = generate_token();
    info!("Token for the client: {token}");
    let token_string = SecretString::from(format!("{non_existing_client}:{token}"));
    let expected_error = check_access(&test_app.db_pool, token_string).await;
    assert!(expected_error.is_err());
    match expected_error {
        Ok(_) => info!("Cant' really be here..."),
        Err(e) => match e.downcast_ref() {
            Some(DataDomainError::InvalidId) => info!("DataDomainError::InvalidId received"),
            _ => panic!("Unexpected error type received"),
        },
    }
    info!("Non existing client_id check passed");
    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}

/// This time, we'll test the code that checks whether a client hash access to the API.
#[actix_web::test]
async fn api_access_by_token() {
    let test_app = spawn_app().await;

    let client_id = seed_api_client(&test_app.db_pool)
        .await
        .expect("Failed to seed an ApiClient into the DB");

    info!("ClientID seeded: {client_id}");

    let plain_token = generate_token();
    let token_string = SecretString::from(format!("{client_id}:{plain_token}"));
    let token_hashed = generate_new_token_hash(SecretString::from(plain_token))
        .expect("Failed to generate the token hash");
    let mut transaction = test_app
        .db_pool
        .begin()
        .await
        .expect("Failed to begin a new DB transaction");
    store_validation_token(
        &mut transaction,
        &token_hashed,
        TimeDelta::days(1),
        &client_id,
    )
    .await
    .expect("Failed to store the token in the DB");
    validate_client_account(&mut transaction, &client_id)
        .await
        .expect("Failed to validate the test client in the DB");
    transaction
        .commit()
        .await
        .expect("Failed to commit transaction to the DB");

    // Yet, the client's account is disabled.
    assert!(check_access(&test_app.db_pool, token_string.clone())
        .await
        .is_err());
    info!("Disabled account check passed");

    // Let's enable it.
    enable_client(&test_app.db_pool, &client_id)
        .await
        .expect("Failed to enable the test client in the DB");
    // Time to have access.
    match check_access(&test_app.db_pool, token_string.clone()).await {
        Ok(_) => info!("Enabled account check passed"),
        Err(e) => {
            error!("{e}");
            panic!("Access was expected")
        }
    }

    // Finally, let's play with the expiry dates.
    let new_expiry_date = Local::now() - TimeDelta::days(2);
    test_app
        .db_pool
        .execute(sqlx::query!(
            "UPDATE ApiToken SET valid_until = ? WHERE client_id = ?;",
            new_expiry_date,
            client_id.to_string()
        ))
        .await
        .expect("Failed to change the expiry date");
    assert!(check_access(&test_app.db_pool, token_string).await.is_err());
    info!("Expiry date check passed");

    // This avoids a dummy warning message in the tracer.
    test_app.db_pool.close().await;
}
