// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Functions related to sending emails using [MailjetClient].

use crate::domain::{ClientId, ServerError};
use actix_web::web::Data;
use mailjet_client::{data_objects, MailjetClient};
use tracing::{debug, error, info};

#[tracing::instrument(skip(mail_client, confirmation_link))]
pub async fn send_confirmation_email(
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
            include_str!("./templates/confirmation_email.txt"),
            confirmation_link
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

#[tracing::instrument(skip(mail_client))]
pub async fn notify_pending_req(
    mail_client: Data<MailjetClient>,
    id: &ClientId,
) -> Result<(), ServerError> {
    let mail = data_objects::MessageBuilder::default()
    .with_from(
        mail_client
            .email_address
            .as_deref()
            .expect("Missing email address of the backend service"),
        mail_client.email_name.as_deref(),
    )
    .with_to(
        mail_client
            .email_address
            .as_deref()
            .expect("Missing email address of the backend service"),
        mail_client.email_name.as_deref(),
    )
    .with_subject("New client of the API validated")
    .with_text_body(
        &format!("A new client ({id}) has validated the account. Proceed to the evaluation of the request.")
    )
    .build();

    let mail_req = data_objects::SendEmailParams {
        sandbox_mode: Some(false),
        advance_error_handling: Some(false),
        globals: None,
        messages: Vec::from([mail]),
    };

    match mail_client.send_email(&mail_req).await {
        Ok(info) => {
            info!("Email sent to the admin");
            debug!("{:?}", info);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email to the admin ({e})");
            Err(ServerError::EmailClientError)
        }
    }
}
