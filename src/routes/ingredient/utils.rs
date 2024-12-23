// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::{Ingredient, ServerError};
use sqlx::MySqlPool;
use std::error::Error;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[instrument(skip(pool, ingredient))]
pub async fn check_ingredient(
    pool: &MySqlPool,
    ingredient: Ingredient,
) -> Result<Vec<Ingredient>, Box<dyn Error>> {
    let rows = sqlx::query!(
        r#"SELECT `id`, `name`, `category`, `description` FROM Ingredient i WHERE i.name like ?"#,
        format!("%{}%", ingredient.name()),
    )
    .fetch_all(pool)
    .await?;

    let mut ingredients = Vec::new();
    for r in rows {
        ingredients.push(Ingredient::parse(
            Some(&r.id),
            r.name.as_str(),
            r.category.as_str(),
            r.description.as_deref(),
        )?);
    }

    Ok(ingredients)
}

#[instrument(skip(pool, id))]
pub async fn get_ingredient_from_db(
    pool: &MySqlPool,
    id: &Uuid,
) -> Result<Option<Ingredient>, Box<dyn Error>> {
    let row = sqlx::query!(
        r#"SELECT `id`, `name`, `category`, `description`
        FROM `Ingredient` WHERE `id`=?"#,
        id.to_string()
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let raw_ingredient = match row {
        Some(i) => i,
        None => {
            return {
                info!("No ingredient was found with the ID: {id}");
                Ok(None)
            }
        }
    };

    let ingredient = Ingredient::parse(
        Some(&raw_ingredient.id),
        &raw_ingredient.name,
        &raw_ingredient.category,
        raw_ingredient.description.as_deref(),
    )?;

    Ok(Some(ingredient))
}
