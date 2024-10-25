// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for managing access tokens of the API.

use crate::domain::{ClientId, DataDomainError, ServerError};
use argon2::{
    password_hash::SaltString,
    {Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version},
};
use chrono::{Local, TimeDelta};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Executor, MySql, MySqlPool, Transaction};
use std::{error::Error, str::FromStr};
use tracing::{debug, error, info};

/// Check if a given token matches the hash stored in the DB.
///
/// # Description
///
/// This function receives two values: the stored hash of the token in the DB, and the token used by the client in
/// a request to the API. The function hashes the given token and compares both. If both match, `Ok(())` is returned,
/// and an `Err(InvalidAccessCredentials)` otherwise.
#[tracing::instrument(name = "Validate credentials", skip(expected_token, given_token))]
pub fn verify_token(
    expected_token: SecretString,
    given_token: SecretString,
) -> Result<(), DataDomainError> {
    let expected_token_hash = PasswordHash::new(expected_token.expose_secret()).map_err(|e| {
        error!("Couldn't hash the given password: {e}");
        DataDomainError::InvalidAccessCredentials
    })?;

    match Argon2::default()
        .verify_password(given_token.expose_secret().as_bytes(), &expected_token_hash)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(DataDomainError::InvalidAccessCredentials),
    }
}

/// Generate a token of 25 alphanumeric characters.
pub fn generate_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

/// Hash a plain token using Argon2.
pub fn generate_new_token_hash(plain_token: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let token_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(plain_token.expose_secret().as_bytes(), &salt)
    .map_err(|_| ServerError::DbError)?
    .to_string();

    Ok(SecretString::from(token_hash))
}

/// Store a validation token in the DB.
#[tracing::instrument(skip(transaction, token))]
pub async fn store_validation_token(
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
pub async fn delete_token(pool: &MySqlPool, token: SecretString) -> Result<(), ServerError> {
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

/// Check if the client hash access to the restricted API's endpoints.
///
/// # Description
///
/// Given a client access token, the stored hash of the token is retrieved from the database and compared. If the
/// comparison is positive, it is checked if the client is enabled.
pub async fn check_access(pool: &MySqlPool, token: SecretString) -> Result<(), Box<dyn Error>> {
    // Let's split the token to get the client's ID and the token itself.
    let token_split = token.expose_secret().split(':').collect::<Vec<&str>>();
    let client_id = token_split[0];
    let token = SecretString::from(token_split[1]);
    // First, retrieve the credentials for the client using the email.
    let query = sqlx::query!(
        r#"
        SELECT at.api_token, at.valid_until, au.enabled
        FROM ApiUser au natural join ApiToken at
        WHERE au.id = ?
        "#,
        client_id.to_string()
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        Box::new(ServerError::DbError)
    })?;

    let (token_saved, valid_until, enabled) = match query {
        Some(record) => (
            SecretString::from(record.api_token),
            record.valid_until,
            record.enabled,
        ),
        None => {
            info!("The given client ID ({client_id}) does not exist in the DB");
            return Err(Box::new(DataDomainError::InvalidId));
        }
    };

    // First, check if the given pair client-token matches the saved one. This avoid giving information about disabled
    // accounts or expired tokens to people that has no access to the API.
    verify_token(token_saved, token).map_err(|e| Box::new(e))?;
    debug!("The token is valid and registered to the client");

    // Second, check if the account is actually enabled.
    if enabled.unwrap_or_default() > 0 {
        debug!("The client's account is enabled");
        // Finally, check that the token is not expired.
        if valid_until.date_naive() - Local::now().date_naive() < TimeDelta::zero() {
            debug!("The client's token is expired");
            Err(Box::new(DataDomainError::ExpiredAccess))
        } else {
            Ok(())
        }
    } else {
        Err(Box::new(DataDomainError::AccountDisabled))
    }
}

/// Enable an API client account.
#[tracing::instrument(skip(pool))]
pub async fn enable_client(pool: &MySqlPool, client_id: &ClientId) -> Result<(), ServerError> {
    let query = sqlx::query!(
        r#"
    UPDATE ApiUser SET enabled = TRUE
    WHERE id = ?;
    "#,
        client_id.to_string()
    );

    pool.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(())
}

/// Check if the user attempted to or is registered already in the DB.
#[tracing::instrument(skip(pool))]
pub async fn check_existing_user(
    pool: &MySqlPool,
    email: &str,
) -> Result<ClientId, Box<dyn Error>> {
    let existing_id = sqlx::query!("SELECT id FROM ApiUser WHERE email = ?", email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?;

    match existing_id {
        Some(record) => Ok(ClientId::from_str(&record.id).unwrap()),
        None => Err(Box::new(DataDomainError::InvalidEmail)),
    }
}

// Validate client's account
#[tracing::instrument(skip(transaction))]
pub async fn validate_client_account(
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use secrecy::SecretString;

    #[rstest]
    fn equal_token_hash_match() {
        let token = SecretString::from(generate_token());
        let token_hash =
            generate_new_token_hash(token.clone()).expect("Failed to generate token hash");
        //let token2_hash = generate_new_token_hash(token).expect("Failed to generate token hash");
        assert!(verify_token(token_hash, token).is_ok())
    }

    #[rstest]
    fn different_token_hash_does_not_match() {
        let token = SecretString::from(generate_token());
        let token_hash = generate_new_token_hash(token).expect("Failed to generate token hash");
        let token = SecretString::from(generate_token());
        let token2_hash = generate_new_token_hash(token).expect("Failed to generate token hash");
        assert!(verify_token(token_hash, token2_hash).is_err())
    }
}
