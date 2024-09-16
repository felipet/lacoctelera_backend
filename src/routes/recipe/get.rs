use crate::domain::{Ingredient, Recipe, RecipeCategory, RecipeQuery, Tag};
use actix_web::{get, web, HttpResponse, Responder};
use std::convert::TryFrom;
use std::fmt::Display;
use tracing::info;
use uuid::Uuid;

/// GET method for the /recipe endpoint (Public).
///
/// # Description
///
/// The GET method allows searching a recipe in the DB using any combination of the fields included in `RecipeQuery`.
/// If `RecipeQuery::id` is provided along the request body, any other fields included in the request will be ignored,
/// i.e. this is not a search but a direct attempt to get a particular recipe from the DB, thus up to a single recipe
/// would be returned.
///
/// When using any other combination of fields but the ID, a search will be performed in the DB. Many results might
/// match the given query, so be aware this is not the best way to retrieve a particular recipe from the DB.
///
/// An intersection is applied between all the matches that result from applying a search using each one of the given
/// query fields. For instance, if a `RecipeQuery`` like this is given:
///
/// > RecipeQuery {
/// >   name: "Delicious Cocktail",
/// >   tags: ["non-alcoholic"],
/// >   rating: StarRate::Five,
/// > }
///
/// And no existing recipes in the DB meet all the given attributes, an empty array will be returned despite some of
/// the individual search operations produced some matches.
#[utoipa::path(
    get,
    tag = "Recipe",
    params(RecipeQuery),
    responses(
        (
            status = 200,
            description = "The query was executed successfully",
            body = [Recipe],
            headers(
                ("Access-Control-Allow-Origin"),
                ("Content-Type"),
                ("Cache-Control"),
            )
        ),
        (
            status = 429,
            description = "Too many requests",
            headers(
                ("Access-Control-Allow-Origin"),
                ("Retry-After"),
            )
        ),

    )
)]
#[get("/recipe")]
pub async fn get_recipe(req: web::Query<RecipeQuery>) -> impl Responder {
    let search_type: SearchType = (&req.0).try_into().expect("Wrong query");

    info!("Recipe search ({search_type}) using: {{{}}}", req.0);

    let template_recipe = Recipe::new(
        &Uuid::now_v7().to_string(),
        "Demo recipe",
        None,
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        Some(&Vec::from([
            Tag::new("alcoholic").unwrap(),
            Tag::new("rum-based").unwrap(),
        ])),
        &RecipeCategory::Easy.to_string(),
        Some("A delicious cocktail for summer."),
        None,
        &Vec::from([
            Ingredient::parse("Rum", "spirit", None).unwrap(),
            Ingredient::parse("Pineapple Juice", "other", None).unwrap(),
        ]),
        &["Pour all the ingredients in a shaker", "Shake and serve"],
        &Uuid::now_v7().to_string(),
    )
    .unwrap();

    HttpResponse::NotImplemented().json(template_recipe)
}

#[derive(Debug, Clone)]
enum SearchType {
    ById,
    ByName,
    ByTags,
    ByRating,
    ByCategory,
    Intersection,
}

impl Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ss = match self {
            SearchType::ById => "ById",
            SearchType::ByName => "ByName",
            SearchType::ByTags => "ByTags",
            SearchType::ByRating => "ByRating",
            SearchType::ByCategory => "ByCategory",
            SearchType::Intersection => "Intersection",
        };

        write!(f, "{ss}")
    }
}

fn multiple_choices(query: &RecipeQuery) -> bool {
    if (query.name.is_some()
        && (query.tags.is_some() || query.rating.is_some() || query.category.is_some()))
        || (query.tags.is_some() && (query.rating.is_some() || query.category.is_some()))
        || (query.rating.is_some() && query.category.is_some())
    {
        return true;
    }

    false
}

impl TryFrom<&RecipeQuery> for SearchType {
    type Error = String;

    fn try_from(query: &RecipeQuery) -> std::result::Result<Self, Self::Error> {
        if query.id.is_some() {
            Ok(SearchType::ById)
        } else if multiple_choices(query) {
            Ok(SearchType::Intersection)
        } else if query.name.is_some() {
            Ok(SearchType::ByName)
        } else if query.tags.is_some() {
            Ok(SearchType::ByTags)
        } else if query.rating.is_some() {
            Ok(SearchType::ByRating)
        } else if query.category.is_some() {
            Ok(SearchType::ByCategory)
        } else {
            Err("Invalid conversion".to_string())
        }
    }
}
