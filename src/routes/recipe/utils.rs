// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::{
    DataDomainError, QuantityUnit, Recipe, RecipeCategory, RecipeContains, ServerError, StarRate,
    Tag,
};
use sqlx::{Executor, MySqlPool};
use std::error::Error;
use tracing::{debug, error, info, instrument};
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

#[instrument(skip(pool))]
pub async fn get_recipe_from_db(pool: &MySqlPool, id: &Uuid) -> Result<Recipe, Box<dyn Error>> {
    let row = sqlx::query!("SELECT * FROM `Cocktail` WHERE id=?", id.to_string(),)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?;

    if row.is_none() {
        return Err(Box::new(DataDomainError::InvalidId));
    }

    let record = row.unwrap();

    let (author_tags, tags) = get_tags_for_recipe(pool, id.to_string().as_ref()).await?;
    let ingredients = get_ingredients_for_recipe(pool, id.to_string().as_ref()).await?;

    let recipe = Recipe::new(
        Some(Uuid::parse_str(&record.id).map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?),
        &record.name,
        record.image_id.as_deref(),
        Some(&author_tags),
        Some(&tags),
        match record.category.as_deref() {
            Some(category) => category,
            None => {
                error!("The recipe has no associated category");
                return Err(Box::new(ServerError::DbError));
            }
        },
        record.description.as_deref(),
        record.url.as_deref(),
        &ingredients,
        &stepize(&record.steps),
        record.owner.as_deref(),
    )?;

    Ok(recipe)
}

#[instrument(skip(pool))]
pub async fn search_recipe_by_name(
    pool: &MySqlPool,
    name: &str,
) -> Result<Vec<Uuid>, Box<dyn Error>> {
    let recipes = sqlx::query!(
        r#"SELECT `id` FROM `Cocktail` WHERE name like ?"#,
        &format!("%{name}%"),
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    });

    let mut found_recipes = Vec::new();

    if let Ok(ids) = recipes {
        for id in ids.iter() {
            found_recipes.push(Uuid::parse_str(&id.id).map_err(|_| {
                error!("Failed to parse ID from a value of the DB");
                ServerError::DbError
            })?);
        }

        info!(
            "{} recipes found using the name: {name}",
            found_recipes.len()
        );
        debug!("{:?}", found_recipes);
    } else {
        info!("No recipes found using the name: {name}");
    }

    Ok(found_recipes)
}

#[instrument(skip(pool))]
pub async fn search_recipe_by_category(
    pool: &MySqlPool,
    category: RecipeCategory,
) -> Result<Vec<Uuid>, Box<dyn Error>> {
    let recipes = sqlx::query!(
        r#"SELECT `id` FROM `Cocktail` WHERE `category`=?"#,
        &category.to_string(),
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    });

    let mut found_recipes = Vec::new();

    if let Ok(ids) = recipes {
        for id in ids.iter() {
            found_recipes.push(Uuid::parse_str(&id.id).map_err(|_| {
                error!("Failed to parse ID from a value of the DB");
                ServerError::DbError
            })?);
        }

        info!(
            "{} recipes found using the category: {category}.",
            found_recipes.len()
        );
        debug!("{:?}", found_recipes);
    } else {
        info!("No recipes found using the category: {category}.");
    }

    Ok(found_recipes)
}

#[instrument(skip(pool))]
pub async fn search_recipe_by_rating(
    pool: &MySqlPool,
    rating: StarRate,
) -> Result<Vec<Uuid>, Box<dyn Error>> {
    let recipes = sqlx::query!(
        r#"SELECT `id` FROM `Cocktail` WHERE `rating`>=?"#,
        &rating.to_string(),
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    });

    let mut found_recipes = Vec::new();

    if let Ok(ids) = recipes {
        for id in ids.iter() {
            found_recipes.push(Uuid::parse_str(&id.id).map_err(|_| {
                error!("Failed to parse ID from a value of the DB");
                ServerError::DbError
            })?);
        }

        info!(
            "{} recipes found with more than {rating} stars.",
            found_recipes.len()
        );
        debug!("{:?}", found_recipes);
    } else {
        info!("No recipes found having {rating} or more stars.");
    }

    Ok(found_recipes)
}

#[instrument(skip(pool))]
async fn get_tags_for_recipe(
    pool: &MySqlPool,
    id: &str,
) -> Result<(Vec<Tag>, Vec<Tag>), Box<dyn Error>> {
    let records = sqlx::query!(
        "SELECT `tag`, `type` from `Tagged` WHERE `cocktail_id` = ?",
        id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{e}");
        ServerError::DbError
    })?;

    let mut tags = Vec::new();
    let mut author_tags = Vec::new();

    for element in records {
        if element.r#type == "author" {
            author_tags.push(Tag {
                identifier: element.tag,
            });
        } else {
            tags.push(Tag {
                identifier: element.tag,
            });
        }
    }

    Ok((author_tags, tags))
}

#[instrument(skip(pool))]
async fn get_ingredients_for_recipe(
    pool: &MySqlPool,
    id: &str,
) -> Result<Vec<RecipeContains>, Box<dyn Error>> {
    let records = sqlx::query!(
        "SELECT `ingredient_id`, `amount` FROM `UsedIngredient` WHERE `cocktail_id`=?",
        id,
    )
    .fetch_all(pool)
    .await?;

    debug!("Found ingredients: {:?}", records);

    let mut ingredients = Vec::new();

    for row in records {
        let split: Vec<&str> = row.amount.split(" ").collect();
        let quantity = split[0].parse::<f32>().map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?;

        let unit: QuantityUnit = split[1].try_into().map_err(|e| {
            error!("{e}");
            ServerError::DbError
        })?;

        ingredients.push(RecipeContains {
            quantity,
            unit,
            ingredient_id: Uuid::parse_str(&row.ingredient_id).map_err(|e| {
                error!("{e}");
                ServerError::DbError
            })?,
        });
    }

    Ok(ingredients)
}

fn stepize(steps: &str) -> Vec<&str> {
    let mut step_list = Vec::new();

    for line in steps.split("/n") {
        step_list.push(line);
    }

    step_list
}
