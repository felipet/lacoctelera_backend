// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::Ingredient;
use sqlx::{Executor, MySqlPool, Row};
use std::error::Error;
use tracing::{debug, error, instrument};
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
