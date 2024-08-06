use crate::domain::Ingredient;
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize, Debug)]
struct FormData {
    name: String,
    category: String,
    desc: Option<String>,
}

#[post("/ingredient")]
pub async fn add_ingredient(
    ingredient: web::Json<FormData>,
    pool: web::Data<MySqlPool>,
) -> HttpResponse {
    let ingredient = Ingredient::parse(
        &ingredient.name,
        ingredient.category.as_ref(),
        ingredient.desc.as_deref(),
    )
    .expect(&format!(
        "Failed to parse an Ingredient from the Form: {:#?}",
        ingredient
    ));

    insert_ingredient(&pool, ingredient)
        .await
        .expect("Failed to insert ingredient");

    HttpResponse::Ok().finish()
}

async fn insert_ingredient(pool: &MySqlPool, ingredient: Ingredient) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO test_cocktail.Ingredient (name, category, `desc`) VALUES
        (?, ?, ?)
        "#,
        ingredient.name(),
        ingredient.category().to_str().to_owned(),
        ingredient.desc(),
    )
    .execute(pool)
    .await?;

    Ok(())
}
