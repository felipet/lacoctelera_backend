use crate::domain::Ingredient;
use actix_web::{get, web, HttpResponse, Responder, Result};
use sqlx::MySqlPool;

#[get("/ingredient")]
pub async fn get_ingredient(
    pool: web::Data<MySqlPool>,
    req: web::Query<Ingredient>,
) -> impl Responder {
    // First, validate the given form as a correct name for the instantiation of an Ingredient.
    let query_ingredient = match Ingredient::try_from(&req.name[..]) {
        Ok(ingredient) => ingredient,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Issue a query to the DB to search for ingredients using the given name.
    let ingredients = match check_ingredient(&pool, query_ingredient).await {
        Ok(ingredients) => ingredients,
        Err(_) => Vec::new(),
    };

    HttpResponse::Ok().json(ingredients)
}

async fn check_ingredient(
    pool: &MySqlPool,
    ingredient: Ingredient,
) -> Result<Vec<Ingredient>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"SELECT `name`, `category`, `desc` FROM test_cocktail.Ingredient i WHERE i.name like ?"#,
        format!("%{}%", ingredient.name),
    )
    .fetch_all(pool)
    .await?;

    let ingredients = rows
        .iter()
        .map(|r| Ingredient {
            name: r.name.clone(),
            category: Some(r.category.clone().into()),
            desc: r.desc.clone(),
        })
        .collect();

    Ok(ingredients)
}
