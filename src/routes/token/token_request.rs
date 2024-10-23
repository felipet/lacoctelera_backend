//! Request a new API token for the restricted endpoints.

use crate::domain::{auth::TokenRequestData, ServerError};
use actix_web::{
    get, http::header::ContentType, post, web, web::Data, web::Form, HttpRequest, HttpResponse,
    Responder,
};
use chrono::Local;
use mailjet_client::{data_objects, MailjetClient};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Executor, MySql, MySqlPool, Transaction};
use tracing::{debug, error, info};
use uuid::Uuid;

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
        .body(include_str!("token_request.html"))
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
            let token = generate_token();
            store_validation_token(&mut transaction, &token, &client_id)
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
                "The given email is already registered in the system. If you have any issue to use your existing API \
                token, please contact the system administrator: {}",
                mail_client
                    .email_address
                    .as_deref()
                    .expect("Missing email address of the system administrator")
            )));
        }
    };

    // Compose the confirmation link.
    let link = format!(
        "{}/validate?email={}&token={token}",
        req.full_url(),
        form.email()
    );

    // Finally, send the confirmation email to the recipient.
    match send_confirmation_email(mail_client, &link, form.email()).await {
        Ok(_) => Ok(HttpResponse::Accepted()
            .body("Please, check your email's inbox and confirm your request.")),
        Err(_) => Ok(HttpResponse::InternalServerError()
            .body("Something went wrong, please try again later.")),
    }
}

#[tracing::instrument(skip(mail_client))]
async fn send_confirmation_email(
    mail_client: Data<MailjetClient>,
    confirmation_link: &str,
    recipient: &str,
) -> Result<(), ServerError> {
    // Build a new message.
    let mail = data_objects::MessageBuilder::default()
        .with_from(
            mail_client
                .email_address
                .as_deref()
                .expect("Missing email address of the backend service"),
            mail_client.email_name.as_deref(),
        )
        .with_to(recipient, None)
        .with_text_body(&format!(
            r#"Greetings from La Coctelera!
You are receiving this email because a token request to access the API of La Coctelera was received.
If you don't recognize this service, feel free to ignore this message.
To complete the request process, please, visit the following link to validate your email: {confirmation_link}
        "#
        ))
        .with_subject("Verify your email")
        .build();

    let mail_req = data_objects::SendEmailParams {
        sandbox_mode: Some(false),
        advance_error_handling: Some(false),
        globals: None,
        messages: Vec::from([mail]),
    };

    match mail_client.send_email(&mail_req).await {
        Ok(info) => {
            info!("Email sent to {recipient}");
            debug!("{:?}", info);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email to {recipient} ({e})");
            Err(ServerError::EmailClientError)
        }
    }
}

#[post("/validate")]
pub async fn token_validation_post(mail_client: Data<MailjetClient>) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// Generate a token
fn generate_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

/// Check if the user attempted to or is registered already in the DB.
#[tracing::instrument(
    name = "Check if an email was registered before in the DB"
    skip(pool)
)]
async fn check_existing_user(pool: &MySqlPool, email: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let existing_id = sqlx::query!("SELECT id FROM ApiUser WHERE email = ?", email)
        .fetch_optional(pool)
        .await?;

    match existing_id {
        Some(record) => Ok(Some(Uuid::parse_str(&record.id).unwrap())),
        None => Ok(None),
    }
}

/// Register a new request in the DB with status = `pending`
#[tracing::instrument(
    name = "Add a new entry in the DB for a new API token request"
    skip(transaction)
)]
async fn register_new_request(
    transaction: &mut Transaction<'static, MySql>,
    form: &TokenRequestData,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
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

    transaction.execute(query).await?;

    Ok(id)
}

/// Store a validation token in the DB.
#[tracing::instrument(
    name = "Link a validation token with a client ID"
    skip(transaction, token)
)]
async fn store_validation_token(
    transaction: &mut Transaction<'static, MySql>,
    token: &str,
    client_id: &Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO ApiToken
        (created, api_token, valid_until, client_id)
        VALUES(current_timestamp(), ?, ?, ?);
        "#,
        token,
        Local::now(),
        client_id.to_string(),
    );

    transaction.execute(query).await?;

    Ok(())
}
