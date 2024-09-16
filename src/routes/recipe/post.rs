use crate::domain::Recipe;
use actix_web::{post, web, HttpResponse};
use tracing::info;

/// POST method for /recipe endpoint (Restricted)
#[utoipa::path(
    post,
    tag = "Recipe",
    security(
        ("api_key" = [])
    )
)]
#[post("/recipe")]
pub async fn post_recipe(req: web::Json<Recipe>) -> HttpResponse {
    info!("Post new recipe: {:#?}", req.0);

    HttpResponse::NotImplemented().finish()
}
