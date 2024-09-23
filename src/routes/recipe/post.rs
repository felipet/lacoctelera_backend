use crate::domain::Recipe;
use actix_web::{post, web, HttpResponse};
use tracing::info;

/// POST method for the /recipe endpoint (Restricted)
///
/// # Description
///
/// This method creates new recipes in the DB using the data provided by authors. Recipes are identified by an unique
/// ID that is generated by the backend sw before inserting a recipe into the DB. This means that the same recipe
/// can be pushed several times either by the same author or by another one.
///
/// A previous search in the DB is advised to avoid having many similar recipes. However, every author is free to
/// add a recipe that is quite similar to another one if he/she likes to.
///
/// A new recipe shall populate all the fields marked with a red asterisk in the recipe's schema. Fields that are
/// optional (not marked with the red asterisk) are, most likely, being filled by the backend logic. However, the
/// following optional fields are meant to be populated by the author, but they are not mandatory:
/// - *author_tags*: Tags that can be freely assigned by the author.
/// - *description*: A free text input in which the author can describe in detail the recipe.
/// - *url*: Useful to link the recipe entry to another web resource.
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