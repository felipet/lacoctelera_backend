// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::{DataDomainError, Recipe, ServerError, Tag};
use sqlx::{Executor, MySqlPool};
use std::error::Error;
use tracing::{error, instrument};
use uuid::Uuid;

#[instrument(skip(pool))]
pub async fn register_new_recipe(
    pool: &MySqlPool,
    recipe: &Recipe,
) -> Result<Uuid, Box<dyn Error>> {
    // First, let's handle tags. If tags are already defined in the system, add a new entry in the `Tagged` table.
    // Otherwise, register the new tag, and add the entry in `Tagged`.

    if let Some(tags) = recipe.tags() {
        for tag in tags {
            sqlx::query!(
                "INSERT IGNORE INTO `Tag` SET `identifier` = ?",
                tag.identifier
            )
            .execute(pool)
            .await
            .map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?;
        }
    }

    if let Some(tags) = recipe.author_tags() {
        for tag in tags {
            sqlx::query!(
                "INSERT IGNORE INTO `Tag` SET `identifier` = ?",
                tag.identifier
            )
            .execute(pool)
            .await
            .map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?;
        }
    }

    let new_id = Uuid::now_v7();

    let mut transaction = pool.begin().await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let query = sqlx::query!(
        r#"INSERT INTO `Cocktail` (`id`, `name`, `description`, `category`, `image_id`, `url`, `rating`, `owner`, `steps`)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        new_id.to_string(),
        recipe.name(),
        recipe.description(),
        recipe.category().to_string(),
        recipe.image_id(),
        recipe.url(),
        recipe.rating().to_string(),
        recipe.owner().map(|s| s.to_string()),
        recipe.steps().join("/n"),
    );

    transaction.execute(query).await.map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    for ingredient in recipe.ingredients() {
        transaction
            .execute(sqlx::query!(
                "INSERT INTO `UsedIngredient` (`cocktail_id`, `ingredient_id`, `amount`) VALUES (?, ?, ?)",
                new_id.to_string(),
                ingredient.ingredient_id.to_string(),
                &format!("{} {}", ingredient.quantity, ingredient.unit.to_string()),
            ))
            .await
            .map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?;
    }

    if let Some(tags) = recipe.author_tags() {
        for tag in tags {
            transaction
                .execute(sqlx::query!(
                    "INSERT INTO `Tagged` (`id`, `cocktail_id`, `type`, `tag`) VALUES (?, ?, ?, ?)",
                    Uuid::now_v7().to_string(),
                    new_id.to_string(),
                    "author",
                    tag.identifier,
                ))
                .await
                .map_err(|e| {
                    error!("{e}");
                    ServerError::DbError
                })?;
        }
    }

    if let Some(tags) = recipe.tags() {
        for tag in tags {
            transaction
                .execute(sqlx::query!(
                    "INSERT INTO `Tagged` (`id`, `cocktail_id`, `type`, `tag`) VALUES (?, ?, ?, ?)",
                    Uuid::now_v7().to_string(),
                    new_id.to_string(),
                    "backend",
                    tag.identifier,
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

    Ok(new_id)
}
