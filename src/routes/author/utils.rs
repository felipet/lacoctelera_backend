// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    domain::{Author, DataDomainError, ServerError, SocialProfile},
    routes::author::get::QueryData,
};
use names::Generator;
use sqlx::{Executor, MySqlPool, Row};
use std::error::Error;
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
    let mut transaction = pool.begin().await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let query = sqlx::query!(
        "INSERT INTO Author VALUES (?, ?, ?, ?, ?, ?, ?);",
        id,
        name,
        surname,
        author.email(),
        author.shareable(),
        author.description(),
        author.website(),
    );

    transaction.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    // If the author hash any social profile, add the entry in the DB.
    if author.social_profiles().is_some() {
        for social_profile in author.social_profiles().unwrap() {
            // Let's try to extract only the user name. If the full URL is given, get the latest breadcrumb.
            let user_account = extract_profile_account(&social_profile.website);

            transaction
                .execute(sqlx::query!(
                    "INSERT INTO AuthorHashSocialProfile (provider_name, user_name, author_id) VALUES (?,?,?);",
                    social_profile.provider_name,
                    user_account,
                    id,
                ))
                .await
                .map_err(|e| {
                    error!("{e}");
                    ServerError::DbError
                })?;
        }
    }

    transaction.commit().await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(Uuid::parse_str(&id).unwrap())
}

#[instrument(skip(pool))]
pub async fn get_author_from_db(
    pool: &MySqlPool,
    author_id: &str,
) -> Result<Author, Box<dyn Error>> {
    let record = sqlx::query!(
        r#"
            SELECT id, name, surname, email, shareable, description, website
            FROM Author
            WHERE id = ?;
            "#,
        author_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let social_profiles = if record.is_some() {
        Some(author_social_profiles(pool, author_id).await?)
    } else {
        None
    };

    let author = if let Some(author) = record {
        Author::new(
            Some(author.id),
            Some(author.name),
            Some(author.surname),
            Some(author.email),
            match author.shareable {
                Some(0) => Some(false),
                _ => Some(true),
            },
            author.description,
            author.website,
            social_profiles.as_deref(),
        )
    } else {
        Err(DataDomainError::InvalidId)
    };

    match author {
        Ok(author) => Ok(author),
        Err(e) => {
            error!("{e}");
            Err(Box::new(e))
        }
    }
}

#[instrument(skip(pool))]
pub async fn search_author_from_db(
    pool: &MySqlPool,
    search_string: QueryData,
) -> Result<Vec<Author>, Box<dyn Error>> {
    let mut found_authors = Vec::new();

    // Obtain the highest priority token for the search.
    let (query, value) = search_string.search_token()?;
    // Compose the query string.
    let query = format!(
        r#"
    SELECT id, name, surname, email, shareable, description, website
    FROM Author
    WHERE {query} = ?"#
    );

    debug!("Searching author using: {value}");

    let query_result = sqlx::query(&query)
        .bind(value)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?;

    for row in query_result {
        debug!("Author found: {:?}", row);
        let author: Result<Author, DataDomainError> = Author::new(
            row.try_get("id").unwrap(),
            row.try_get("name").unwrap(),
            row.try_get("surname").unwrap(),
            row.try_get("email").unwrap(),
            match row.try_get("shareable") {
                Ok(0) => Some(false),
                _ => Some(true),
            },
            row.try_get("description").unwrap(),
            row.try_get("website").unwrap(),
            Some(&Vec::new()),
        );

        debug!("Author: {:?}", author);

        let author = match author {
            Ok(author) => author,
            Err(e) => {
                error!("{e}");
                return Err(Box::new(ServerError::DbError));
            }
        };

        found_authors.push(author);
    }

    Ok(found_authors)
}

#[instrument(skip(pool))]
pub async fn modify_author_from_db(
    pool: &MySqlPool,
    author: &Author,
) -> Result<(), Box<dyn Error>> {
    let query = sqlx::query!(
        r#"UPDATE Author
        SET name = ?, surname = ?, email = ?, shareable = ?, description = ?, website = ?
        WHERE id = ?
        "#,
        author.name(),
        author.surname(),
        author.email(),
        author.shareable(),
        author.description(),
        author.website(),
        author.id(),
    );

    let mut transaction = pool.begin().await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    transaction.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    if author.social_profiles().is_some() {
        for social_profile in author.social_profiles().unwrap() {
            // Let's try to extract only the user name. If the full URL is given, get the latest breadcrumb.
            let user_account = extract_profile_account(&social_profile.website);

            transaction
                .execute(sqlx::query!(
                    r#"UPDATE AuthorHashSocialProfile
                SET user_name = ?
                WHERE provider_name = ? AND author_id = ?"#,
                    user_account,
                    social_profile.provider_name,
                    author.id(),
                ))
                .await
                .map_err(|e| {
                    error!("{e}");
                    ServerError::DbError
                })?;
        }
    }

    transaction.commit().await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(())
}

#[instrument(skip(pool))]
pub async fn delete_author_from_db(pool: &MySqlPool, author_id: &Uuid) -> Result<(), ServerError> {
    sqlx::query!(
        r#"
        DELETE FROM Author
        WHERE id = ?
        "#,
        author_id.to_string()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    Ok(())
}

#[instrument(skip(pool))]
async fn author_social_profiles(
    pool: &MySqlPool,
    author_id: &str,
) -> Result<Vec<SocialProfile>, ServerError> {
    let records = sqlx::query!(
        r#"
        SELECT provider_name, user_name, website
        FROM AuthorHashSocialProfile ahsp natural join SocialProfile sp
        WHERE ahsp.author_id = ?
        "#,
        author_id.to_string()
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let mut profiles: Vec<SocialProfile> = Vec::new();
    for record in records {
        profiles.push(SocialProfile {
            provider_name: record.provider_name,
            website: format!("{}{}", record.website, record.user_name),
        });
    }

    Ok(profiles)
}

fn extract_profile_account(profile_url: &str) -> &str {
    if profile_url.contains('/') {
        profile_url
            .split("/")
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    } else {
        profile_url
    }
}
