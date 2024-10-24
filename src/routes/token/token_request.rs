//! Request a new API token for the restricted endpoints.

use crate::{
    authentication::{generate_new_token_hash, generate_token},
    domain::{auth::TokenRequestData, ClientId, DataDomainError, ServerError},
    utils::mailing::{notify_pending_req, send_confirmation_email},
};
use actix_web::{
    get, http::header::ContentType, post, web, web::Data, web::Form, HttpRequest, HttpResponse,
    Responder,
};
use anyhow::Context;
use chrono::{DateTime, Local, TimeDelta};
use mailjet_client::MailjetClient;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::{Executor, MySql, MySqlPool, Transaction};
use std::{error::Error, str::FromStr};
use tracing::{debug, error, info};

/// Payload of the token validation POST.
#[derive(Deserialize, Debug)]
struct TokenValidationData {
    pub email: String,
    pub token: String,
}

/// GET for the API's /token/request endpoint.
///
/// # Description
///
/// This endpoint offers a simple HTML form that allows clients interested in accessing the restricted endpoints to
/// request an API token.
#[get("/request")]
pub async fn token_req_get() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../../static/token_request.html"))
}

/// POST for the API's /token/request endpoint.
///
/// # Description
///
/// Once a client fills the requested data, a confirmation email is sent to the given email address. If the email gets
/// confirmed, the request gets actually registered in the system, and waits until the sysadmin approves or rejects it.
#[tracing::instrument(skip(req, form, pool, mail_client))]
#[post("/request")]
pub async fn token_req_post(
    req: HttpRequest,
    form: Form<TokenRequestData>,
    pool: Data<MySqlPool>,
    mail_client: Data<MailjetClient>,
) -> Result<HttpResponse, ServerError> {
    info!("An API token was requested by {}", form.email());

    // Check if the client is already registered in the DB.
    let user_id = match check_existing_user(&pool, form.email()).await {
        Ok(id) => id,
        Err(e) => {
            error!("{e}");
            return Err(ServerError::DbError);
        }
    };

    let token = match user_id {
        // Process to register a new request in the DB.
        None => {
            debug!("The given email was not registered in the DB");
            let mut transaction = pool.begin().await.map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?;
            let client_id = register_new_request(&mut transaction, &form)
                .await
                .map_err(|e| {
                    error!("{e}");
                    ServerError::DbError
                })?;
            let token = SecretString::from(generate_token());
            // Store the temporal validation token with an expiry of 1 day.
            store_validation_token(&mut transaction, &token, TimeDelta::days(1), &client_id)
                .await
                .map_err(|e| {
                    error!("{e}");
                    ServerError::DbError
                })?;
            transaction.commit().await.map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?;

            token
        }
        // Ignore the request.
        Some(id) => {
            info!("The client is already registered in the system ({id})");
            return Ok(HttpResponse::Ok().body(format!(
                include_str!("../../../static/message_template.html"),
                include_str!("../../../static/resources/style.css"),
                "<h3>The given email is already registered in the system.</h3> \
                <h4>If you have any issue to use your existing API token, please contact the system administrator
                </h4>")),
            );
        }
    };

    // Compose the confirmation link.
    let link = format!(
        "{}/validate?email={}&token={}",
        req.full_url(),
        form.email(),
        token.expose_secret(),
    );

    // Finally, send the confirmation email to the recipient.
    send_confirmation_email(mail_client, &link, form.email()).await?;

    Ok(HttpResponse::Accepted().body(format!(
        include_str!("../../../static/message_template.html"),
        include_str!("../../../static/resources/style.css"),
        "<h3>Please, check your email's inbox and confirm your request.</h3>"
    )))
}

/// Endpoint to validate a token request sent to an email account.
///
/// # Description
///
/// This endpoint receives the token that was sent when a client registered a new request using `/token/request`, and
/// if the token matches the stored in the DB, the client receives a new token that is shown only once and stored in
/// the DB (replacing the previous one). This way, only the client knows the token.
#[tracing::instrument(skip(req, pool))]
#[get("/request/validate")]
pub async fn req_validation(
    req: web::Query<TokenValidationData>,
    pool: Data<MySqlPool>,
    mail_client: Data<MailjetClient>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // First, check if the token is valid and received in time.
    let client_id = check_email_validation(&pool, &req.token, &req.email).await?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a connection from the pool")?;

    // Generate a new token using the client's ID and a new random token.
    let token = SecretString::from(generate_token());
    // Show this to the client. It will be gone for ever.
    let token_string = format!("{}:{}", client_id, token.expose_secret());
    // Hash the token part, as that is what we'll store in the DB.
    let token_hashed = generate_new_token_hash(token)?;
    // Store the new token.
    store_validation_token(
        &mut transaction,
        &token_hashed,
        TimeDelta::days(100),
        &client_id,
    )
    .await?;
    validate_client_account(&mut transaction, &client_id).await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new client's access token")?;

    notify_pending_req(mail_client, &client_id).await?;

    let message = format!(
        r#"
        <h3>This is your access token for the API: {token_string}</h3>
        <h4>After this page gets closed, it will be impossible to recover it, save it in a secure place!</h4>
        <h3>However, your account will remain disabled until your request gets approved.</h3>
        <h4>You'll receive an email soon.</h4>
        "#
    );

    Ok(HttpResponse::Accepted().body(format!(
        include_str!("../../../static/message_template.html"),
        include_str!("../../../static/resources/style.css"),
        message
    )))
}

/// Check if the user attempted to or is registered already in the DB.
#[tracing::instrument(skip(pool))]
async fn check_existing_user(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<ClientId>, sqlx::Error> {
    let existing_id = sqlx::query!("SELECT id FROM ApiUser WHERE email = ?", email)
        .fetch_optional(pool)
        .await?;

    match existing_id {
        Some(record) => Ok(Some(ClientId::from_str(&record.id).unwrap())),
        None => Ok(None),
    }
}

/// Register a new request in the DB.
#[tracing::instrument(skip(transaction, form))]
async fn register_new_request(
    transaction: &mut Transaction<'static, MySql>,
    form: &TokenRequestData,
) -> Result<ClientId, ServerError> {
    let id = ClientId::new();
    let query = sqlx::query!(
        r#"
        INSERT INTO ApiUser (id, name,email,validated,enabled,explanation) VALUES
        (?, ?, ?, 0, 0, ?);
        "#,
        id.to_string(),
        form.name(),
        form.email(),
        form.explanation(),
    );

    transaction.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(id)
}

/// Store a validation token in the DB.
#[tracing::instrument(skip(transaction, token))]
async fn store_validation_token(
    transaction: &mut Transaction<'static, MySql>,
    token: &SecretString,
    expiry: TimeDelta,
    client_id: &ClientId,
) -> Result<(), ServerError> {
    let query = sqlx::query!(
        r#"
        INSERT INTO ApiToken
        (created, api_token, valid_until, client_id)
        VALUES(current_timestamp(), ?, ?, ?);
        "#,
        token.expose_secret(),
        Local::now() + expiry,
        client_id.to_string(),
    );

    transaction.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(())
}

/// Delete a token that will be no longer used.
#[tracing::instrument(skip(pool, token))]
async fn delete_token(pool: &MySqlPool, token: SecretString) -> Result<(), ServerError> {
    let query = sqlx::query!(
        "DELETE FROM ApiToken WHERE api_token = ?",
        token.expose_secret()
    );

    pool.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(())
}

// Validate a pair email-token
#[tracing::instrument(
    name = "Validate a token request"
    skip(pool, token, client_email)
)]
async fn check_email_validation(
    pool: &MySqlPool,
    token: &str,
    client_email: &str,
) -> Result<ClientId, Box<dyn Error>> {
    // First, retrieve the credentials for the client using the email.
    let query = sqlx::query!(
        r#"
        SELECT at.client_id, at.valid_until, at.api_token
        FROM ApiUser au natural join ApiToken at
        WHERE au.email = ?
        "#,
        client_email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let record = match query {
        Some(record) => record,
        None => return Err(Box::new(ServerError::DbError)),
    };

    // Given token does not match the saved one, reject the validation process.
    if record.api_token != token {
        info!(
            "The given access token for the client {} is not valid",
            &record.client_id
        );
        Err(Box::new(DataDomainError::InvalidAccessCredentials))
    } else {
        // Ensure the validation took place within the valid time frame.
        let valid_until = match record.valid_until.to_string().parse::<DateTime<Local>>() {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to read valid_until date from the DB: {e}");
                return Err(Box::new(ServerError::DbError));
            }
        };
        if (valid_until - Local::now()) < TimeDelta::days(1) {
            info!("Validation received in time");
            Ok(ClientId::from_str(&record.client_id)
                .expect("Failed to parse ClientId from DB client's ID"))
        } else {
            Err(Box::new(ServerError::DbError))
        }
    }
}

// Validate client's account
#[tracing::instrument(skip(transaction))]
async fn validate_client_account(
    transaction: &mut Transaction<'static, MySql>,
    id: &ClientId,
) -> Result<(), ServerError> {
    let query = sqlx::query!(
        r#"
        UPDATE ApiUser
        SET validated = TRUE
        WHERE id = ?;
        "#,
        id.to_string()
    );

    transaction.execute(query).await.map_err(|e| {
        error!("Error found while updating ApiUser's entry for {id}: {e}");
        ServerError::DbError
    })?;

    Ok(())
}
