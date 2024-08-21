use crate::domain::Ingredient;
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tracing::{debug, error, info, instrument};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FormData {
    pub name: String,
    pub category: String,
    pub desc: Option<String>,
}

/// POST for the API's /ingredient endpoint.
#[utoipa::path(
    post,
    path = "/ingredient",
    tag = "Ingredient",
    request_body(
        content = FormData, description = "The data to register a new Ingredient into the DB",
        example = json!({"name": "vodka", "category": "spirit"})
    ),
    responses(
        (
            status = 200,
            description = "The new ingredient was inserted into the DB successfully"
        ),
        (
            status = 400,
            description = "Format error found in the given JSON",
        ),
        (
            status = 500,
            description = "Broken link to the DB server",
        )
    )
)]
#[instrument(
    target = "lacoctelera::ingredient_post",
    skip(pool, ingredient),
    fields(
        ingredient_name = %ingredient.name,
        ingredient_category = %ingredient.category,
        ingredient_desc = %ingredient.desc.as_deref().unwrap_or_default()
    )
)]
#[post("/ingredient")]
pub async fn add_ingredient(
    ingredient: web::Json<FormData>,
    pool: web::Data<MySqlPool>,
) -> HttpResponse {
    let ingredient = match Ingredient::parse(
        &ingredient.name,
        ingredient.category.as_ref(),
        ingredient.desc.as_deref(),
    ) {
        Ok(ingredient) => {
            debug!("Received JSON parsed as {:#?}", ingredient);
            ingredient
        }
        Err(e) => {
            debug!("Received JSON could not be parsed as an ingredient.");
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

    match insert_ingredient(&pool, ingredient).await {
        Ok(_) => HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .finish(),
        Err(e) => {
            error!("The ingredient could not be inserted in the DB: {e}");
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[instrument(skip(pool, ingredient))]
async fn insert_ingredient(pool: &MySqlPool, ingredient: Ingredient) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO Ingredient (name, category, `desc`) VALUES
        (?, ?, ?)
        "#,
        ingredient.name(),
        ingredient.category().to_str().to_owned(),
        ingredient.desc(),
    )
    .execute(pool)
    .await?;

    info!("New ingredient inserted in the DB.");

    Ok(())
}
