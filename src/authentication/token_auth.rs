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
use chrono::{DateTime, Local, TimeDelta};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Executor, MySql, MySqlPool, Transaction};
use std::error::Error;
use tracing::error;
use uuid::Uuid;

#[tracing::instrument(
    name = "Validate credentials",
    skip(expected_password_hash, password_candidate)
)]
pub fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<(), DataDomainError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .map_err(|e| {
            error!("Couldn't hash the given password: {e}");
            DataDomainError::InvalidAccessCredentials
        })?;

    match Argon2::default().verify_password(
        password_candidate.expose_secret().as_bytes(),
        &expected_password_hash,
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(DataDomainError::InvalidAccessCredentials),
    }
}

/// Generate a token
pub fn generate_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

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
