//! Data objects related to Recipes.
//!
//! # Description
//!
//! This module includes the definition of the object [Recipe] which is a representation of a recipe entry in the
//! `Cocktail` DB. This object is used to transfer data to/from the DB. When a query to the API related to any
//! operation with recipes is aimed, the object [RecipeQuery] shall be used instead of [Recipe]. The former only
//! includes those [Recipe]'s members that the API implement logic for. Furthermore, members are nullable, so only
//! the aimed member needs to be populated by the client of the API.

use std::str::FromStr;

use crate::{
    domain::{DataDomainError, Ingredient, Tag},
    validate_id,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Object that represents a Recipe of the `Cocktail` data base.
///
/// # Description
///
/// This object is used to insert new data into the DB, or to retrieve data from the DB. Members that are wrapped
/// using an [Option] are allowed to take `Null` values at creation time. The rest of the members must receive some
/// valid data when a new [Recipe] is built.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct Recipe {
    /// ID used as PK in the DB.
    #[validate(custom(function = "validate_id"))]
    #[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    id: Uuid,
    /// Recipe's name. Up to 40 chars.
    #[validate(length(min = 2), length(max = 40))]
    name: String,
    /// Path to an image for the cocktail.
    image_id: Option<String>,
    /// List of tags assigned by the recipe's author.
    author_tags: Option<Vec<Tag>>,
    /// List of tags assigned by the internal logic.
    tags: Option<Vec<Tag>>,
    /// Recipe's category.
    #[validate(custom(function = "validate_id"))]
    #[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    category_id: Uuid,
    /// Recipe's rating.
    rating: StarRate,
    #[validate(length(min = 2), length(max = 400))]
    description: Option<String>,
    /// Linked URL of the recipe. For third-party content.
    #[validate(url)]
    url: Option<String>,
    /// Ingredients of the recipe.
    ingredients: Vec<Ingredient>,
    /// Preparation steps of the cocktail.
    steps: Vec<String>,
    /// When the recipe was registered in the DB.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    creation_date: DateTime<Local>,
    /// Indicate whether the recipe was updated after creation and when.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    update_date: Option<DateTime<Local>>,
    /// Recipe's Author ID.
    #[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    author_id: Uuid,
}

/// Query object for the `Recipe` entity.
///
/// # Description
///
/// This is a subset of the `Recipe`'s members. Any of the included members in this `struct` can be used to perform
/// a search in the `Cocktail` DB. Recipe queries are allowed using a single member or a combination of many. In that
/// case, the intersection set of the result sets for each individual query is returned. Notice that set could be
/// empty if all the result sets are disjoint.
#[derive(Clone, Debug, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct RecipeQuery {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub rating: Option<StarRate>,
    pub category: Option<Uuid>,
}

/// Simple `enum` to represent a 5-star rating system.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum StarRate {
    Null = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl Recipe {
    /// Constructor of the object [Recipe].
    ///
    /// # Description
    ///
    /// This function creates a new instance of [Recipe] using the given arguments. Arguments are checked to detect
    /// invalid values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        image_id: Option<&str>,
        author_tags: Option<&[Tag]>,
        tags: Option<&[Tag]>,
        category_id: &str,
        description: Option<&str>,
        url: Option<&str>,
        ingredients: &[Ingredient],
        steps: &[&str],
        author_id: &str,
    ) -> Result<Self, DataDomainError> {
        let id = match Uuid::from_str(id) {
            Ok(id) => id,
            Err(_) => return Err(DataDomainError::InvalidId),
        };

        let category_id = match Uuid::from_str(category_id) {
            Ok(id) => id,
            Err(_) => return Err(DataDomainError::InvalidId),
        };

        let author_id = match Uuid::from_str(author_id) {
            Ok(id) => id,
            Err(_) => return Err(DataDomainError::InvalidId),
        };

        let recipe = Recipe {
            id,
            name: name.into(),
            image_id: image_id.map(String::from),
            author_tags: author_tags.map(Vec::from),
            tags: tags.map(Vec::from),
            category_id,
            rating: StarRate::Null,
            description: description.map(String::from),
            url: url.map(String::from),
            ingredients: Vec::from(ingredients),
            steps: steps.iter().map(|c| String::from(*c)).collect(),
            author_id,
            creation_date: Local::now(),
            update_date: None,
        };

        match recipe.validate() {
            Ok(_) => Ok(recipe),
            Err(e) => Err(DataDomainError::InvalidParams { source: e }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Ingredient;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use uuid::Uuid;

    struct TemplateRecipe<'a> {
        pub id: String,
        pub name: String,
        pub image_id: Option<String>,
        pub author_tags: Option<Vec<Tag>>,
        pub tags: Option<Vec<Tag>>,
        pub category_id: String,
        pub description: Option<String>,
        pub url: Option<String>,
        pub ingredients: Vec<Ingredient>,
        pub steps: &'a [&'a str],
        pub author_id: String,
    }

    #[fixture]
    fn template_recipe<'a>() -> TemplateRecipe<'a> {
        TemplateRecipe {
            id: Uuid::now_v7().to_string(),
            name: "Demo recipe".into(),
            image_id: None,
            author_tags: Some(Vec::from([
                Tag::new("alcoholic").unwrap(),
                Tag::new("rum-based").unwrap(),
            ])),
            tags: Some(Vec::from([
                Tag::new("alcoholic").unwrap(),
                Tag::new("rum-based").unwrap(),
            ])),
            category_id: Uuid::now_v7().to_string(),
            description: Some("A delicious cocktail for summer.".to_owned()),
            url: None,
            ingredients: Vec::from([
                Ingredient::parse("Rum", "spirit", None).unwrap(),
                Ingredient::parse("Pineapple Juice", "other", None).unwrap(),
            ]),
            steps: &["Pour all the ingredients in a shaker", "Shake and serve"],
            author_id: Uuid::now_v7().to_string(),
        }
    }

    #[rstest]
    fn check_recipe_builds_using_valid_data(template_recipe: TemplateRecipe) {
        let recipe = Recipe::new(
            &template_recipe.id,
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category_id,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            &template_recipe.author_id.to_string(),
        );

        assert!(recipe.is_ok());

        let recipe = recipe.unwrap();

        assert_eq!(recipe.id.to_string(), template_recipe.id);
        assert_eq!(recipe.name, template_recipe.name);
        assert_eq!(
            recipe.image_id.as_deref(),
            template_recipe.image_id.as_deref()
        );
        assert_eq!(recipe.author_tags, template_recipe.author_tags);
        assert_eq!(recipe.tags, template_recipe.tags);
        assert_eq!(
            recipe.category_id.to_string(),
            template_recipe.category_id.to_string()
        );
        assert_eq!(recipe.rating, StarRate::Null);
        assert_eq!(
            recipe.description.as_deref(),
            template_recipe.description.as_deref()
        );
        assert_eq!(recipe.url, template_recipe.url);
        assert_eq!(recipe.ingredients, template_recipe.ingredients);
        assert_eq!(recipe.steps, template_recipe.steps);
        assert_eq!(recipe.update_date, None);
        assert_eq!(recipe.author_id.to_string(), template_recipe.author_id);
    }

    #[rstest]
    fn check_recipe_not_builds_using_invalid_data(template_recipe: TemplateRecipe) {
        // Invalid ID test case
        let recipe = Recipe::new(
            "9113-239aab-39393b",
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category_id,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            &template_recipe.author_id.to_string(),
        );

        assert!(recipe.is_err());

        // Invalid name test case
        let recipe = Recipe::new(
            &template_recipe.id,
            "Very long name that should produce an error",
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category_id,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            &template_recipe.author_id.to_string(),
        );

        assert!(recipe.is_err());

        // Invalid description test case
        let recipe = Recipe::new(
            &template_recipe.id,
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category_id,
            Some(&"An extremely long description".repeat(1000)),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            &template_recipe.author_id.to_string(),
        );

        assert!(recipe.is_err());
    }
}
