use crate::domain::Ingredient;
use actix_web::{get, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use sqlx::MySqlPool;

/// `Struct` QueryData models the expected fields for a query string.
///
/// # Description
///
/// Using a `Struct` rather than a simple `String` as the received data for the Query will leverage
/// the internal parsing logic of the framework. This way, the endpoint handler would only receive
/// valid data, since wrong data is rejected and the request is answered with a code 400 by the
/// framework.
#[derive(Deserialize)]
struct QueryData {
    pub name: String,
}

/// GET handler of the API's `/ingredient` endpoint.
#[get("/ingredient")]
pub async fn get_ingredient(
    pool: web::Data<MySqlPool>,
    req: web::Query<QueryData>,
) -> impl Responder {
    // First, validate the given form as a correct name for the instantiation of an Ingredient.
    let query_ingredient = match Ingredient::parse(&req.name, "other", None) {
        Ok(ingredient) => ingredient,
        Err(e) => return HttpResponse::BadRequest().body(format!("{}", e)),
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
        r#"SELECT `id`, `name`, `category`, `desc` FROM test_cocktail.Ingredient i WHERE i.name like ?"#,
        format!("%{}%", ingredient.name()),
    )
    .fetch_all(pool)
    .await?;

    let ingredients = rows
        .iter()
        .map(|r| {
            Ingredient::parse(r.name.as_str(), r.category.as_str(), r.desc.as_deref())
                .unwrap()
                .build_id(r.id)
        })
        .collect();

    Ok(ingredients)
}
