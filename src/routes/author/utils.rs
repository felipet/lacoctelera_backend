// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use crate::{domain::Author, domain::ServerError};
use names::Generator;
use sqlx::{Executor, MySqlPool};
use tracing::{debug, error, instrument};
use uuid::Uuid;

#[instrument(skip(pool))]
pub async fn register_new_author(pool: &MySqlPool, author: &Author) -> Result<Uuid, ServerError> {
    // Compose a funny name in case the `Author` has no name.
    let funny_name: Vec<String> = Generator::default()
        .next()
        .unwrap()
        .split('-')
        .map(String::from)
        .collect();

    // Values for fields that are optional.
    let id = match author.id() {
        Some(id) => id,
        None => Uuid::now_v7().to_string(),
    };

    let name = match author.name() {
        Some(name) => name,
        None => &funny_name[0],
    };

    let surname = match author.surname() {
        Some(surname) => surname,
        None => &funny_name[1],
    };

    debug!("ID for the new Author entry in the DB: {id}");

    let query = sqlx::query!(
        r#"
        INSERT INTO Author
        VALUES (?, ?, ?, ?, ?, ?, ?);
        "#,
        id,
        name,
        surname,
        author.email(),
        author.shareable,
        author.description(),
        author.website(),
    );

    pool.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(Uuid::from_str(&id).unwrap())
}
