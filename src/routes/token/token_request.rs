//! Request a new API token for the restricted endpoints.
//!
//! # Description
//!
//! This module includes the handlers that automate the process of requesting and delivering API access tokens.
//!
//! ## API Token
//!
//! Some of the API's endpoints are restricted to public access, and require clients to identify. Restricted
//! endpoints require an extra parameter as part of the request: the API token. The token is composed of two
//! components: client's ID and the access token.
//!
//! That token is use by the backend to identify the client, and check whether it is approved to access the restricted
//! endpoints or not.
//!
//! ## API Token Request
//!
//! Anyone interested on using the restricted endpoints needs to request an API token. To ease such process, a
//! specific endpoint is enabled in the backend that serves some simple HTML pages: `/token/request`. That endpoint
//! is accessible via a web browser, and includes a simple form that a client must fill before issuing a token request.
//!
//! The request gets registered in the system, but partially, until the client verifies the used email account. The
//! backend sends an email after registering a new request with a validation link that will be available for a day.
//! The client needs to visit such URL in order to complete the request process because during the validation process,
//! the real API token gets generated. It is shown only once to the client, and the hash gets stored into the DB. If
//! the client fails to complete the validation process, or looses the token, the process needs to be restarted.
//!
//! Once the email gets validated, the request is fully registered and sent to evaluation. The evaluation process is
//! manual and involves the system administrator. The result of the evaluation is notified via email to the client. If
//! the request gets approved, the client is ready to start using the restricted endpoints using the token that was
//! given at the end of the validation process.

use crate::{
    authentication::*,
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
    pub token: SecretString,
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
) -> Result<HttpResponse, Box<dyn Error>> {
    info!("An API token was requested by {}", form.email());

    // Check if the client is already registered in the DB.
    match check_existing_user(&pool, form.email()).await {
        Ok(id) => {
            info!("A client ({id}) is already registered with the given email");
            return Ok(HttpResponse::Found().body(format!(
                include_str!("../../../static/message_template.html"),
                "The email is already registered in the system. Please, contact the sysadmin if you have any problem."
            )));
        }
        Err(e) => match e.downcast_ref() {
            Some(DataDomainError::InvalidEmail) => {
                debug!("The given email was not registered in the DB")
            }
            _ => return Err(e),
        },
    }

    // It's a new client, let's register the new request.
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
#[tracing::instrument(skip(req, pool, mail_client))]
#[get("/request/validate")]
pub async fn req_validation(
    req: web::Query<TokenValidationData>,
    pool: Data<MySqlPool>,
    mail_client: Data<MailjetClient>,
) -> Result<HttpResponse, Box<dyn Error>> {
    // First, check if the token is valid and received in time.
    let client_id = check_email_validation(&pool, &req.token, &req.email).await?;

    // Once here, we are about to complete the validation process and deliver a real token to the client, let's
    // delete the temporal token first.
    delete_token(&pool, req.token.clone()).await?;

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

    Ok(HttpResponse::Accepted().body(format!(
        include_str!("../../../static/secret_token.html"),
        token_string
    )))
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

// Validate a pair email-token
#[tracing::instrument(
    name = "Validate a token request"
    skip(pool, token, client_email)
)]
async fn check_email_validation(
    pool: &MySqlPool,
    token: &SecretString,
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
        None => {
            info!("The given email is not registered in the DB");
            return Err(Box::new(DataDomainError::InvalidEmail));
        }
    };

    // Given token does not match the saved one, reject the validation process.
    if record.api_token != token.expose_secret() {
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
