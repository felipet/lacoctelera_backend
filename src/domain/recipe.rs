// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Data objects related to Recipes.
//!
//! # Description
//!
//! This module includes the definition of the object [Recipe] which is a representation of a recipe entry in the
//! `Cocktail` DB. This object is used to transfer data to/from the DB. When a query to the API related to any
//! operation with recipes is aimed, the object [RecipeQuery] shall be used instead of [Recipe]. The former only
//! includes those [Recipe]'s members that the API implement logic for. Furthermore, members are nullable, so only
//! the aimed member needs to be populated by the client of the API.

use crate::{
    domain::{DataDomainError, Tag},
    validate_id,
};
use chrono::{DateTime, Local};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::error;
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
    /// ID used as PK in the DB. Generated by the backend.
    #[validate(custom(function = "validate_id"))]
    #[schema(example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    id: Option<Uuid>,
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
    category: RecipeCategory,
    /// Recipe's rating.
    rating: Option<StarRate>,
    #[validate(length(min = 2), length(max = 400))]
    description: Option<String>,
    /// Linked URL of the recipe. For third-party content.
    #[validate(url)]
    url: Option<String>,
    /// Ingredients of the recipe.
    ingredients: Vec<RecipeContains>,
    /// Preparation steps of the cocktail.
    steps: Vec<String>,
    /// When the recipe was registered in the DB.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    creation_date: Option<DateTime<Local>>,
    /// Indicate whether the recipe was updated after creation and when.
    #[schema(value_type = String, example = "2025-09-11T08:58:56.121331664+02:00")]
    update_date: Option<DateTime<Local>>,
    /// Recipe's Author ID.
    #[schema(example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    author_id: Option<Uuid>,
}

/// Query object for the `Recipe` entity.
///
/// # Description
///
/// This is a subset of the `Recipe`'s members. Any of the included members in this `struct` can be used to perform
/// a search in the `Cocktail` DB. Recipe queries are allowed using a single member or a combination of many. In that
/// case, the intersection set of the result sets for each individual query is returned. Notice that set could be
/// empty if all the result sets are disjoint.
#[derive(Clone, Debug, Serialize, Deserialize, IntoParams)]
pub struct RecipeQuery {
    pub name: Option<String>,
    #[param(example = "tequila,reposado")]
    pub tags: Option<String>,
    pub rating: Option<StarRate>,
    pub category: Option<RecipeCategory>,
}

/// Simple `enum` to represent a 5-star rating system.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum StarRate {
    #[serde(rename = "0")]
    Null = 0,
    #[serde(rename = "1")]
    One = 1,
    #[serde(rename = "2")]
    Two = 2,
    #[serde(rename = "3")]
    Three = 3,
    #[serde(rename = "4")]
    Four = 4,
    #[serde(rename = "5")]
    Five = 5,
}

impl std::fmt::Display for StarRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StarRate::One => "1",
            StarRate::Two => "2",
            StarRate::Three => "3",
            StarRate::Four => "4",
            StarRate::Five => "5",
            StarRate::Null => "0",
        };

        write!(f, "{s}")
    }
}

impl From<StarRate> for u8 {
    fn from(value: StarRate) -> Self {
        match value {
            StarRate::One => 1,
            StarRate::Two => 2,
            StarRate::Three => 3,
            StarRate::Four => 4,
            StarRate::Five => 5,
            _ => 0,
        }
    }
}

/// Categories of recipes.
///
/// # Description
///
/// Recipes are categorized using the degree of difficult of the preparation process of the recipe.
///
/// - Category [RecipeCategory::Easy] should include recipes that need no special equipment nor uncommon spirits.
/// - Category [RecipeCategory::Medium] should include recipes that need basic equipment but use common spirits.
/// - Category [RecipeCategory::Advanced] should include recipes that need specific equipment and might use uncommon
///   spirits.
/// - Cageory [RecipeCategory::Pro] should include advanced recipes that need complicated preparation techniques or
///   very special equipment.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecipeCategory {
    Easy,
    Medium,
    Advanced,
    Pro,
}

/// Object that represents the relation between [Ingredient] and [Recipe].
///
/// # Description
///
/// This object implements the relation [Recipe] contains [Ingredient] with an attribute that specifies the quantity.
/// When a new recipe is created, ingredients are added to it in concrete amounts. Several types of units are given
/// to clients using [QuantityUnit]. This way, clients can easily introduce recipes using the units they are most
/// comfortable with.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RecipeContains {
    pub quantity: f32,
    pub unit: QuantityUnit,
    pub ingredient_id: Uuid,
}

/// `Enum` type that defines common types of units in cooking recipes.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum QuantityUnit {
    #[serde(rename = "g")]
    Grams,
    #[serde(rename = "ml")]
    MilliLiter,
    Dash,
    Unit,
    #[serde(rename = "oz")]
    Ounces,
    Drops,
    #[serde(rename = "tbsp")]
    TableSpoon,
    #[serde(rename = "tsp")]
    TeaSpoon,
    Cups,
}

impl fmt::Display for QuantityUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QuantityUnit::Grams => "g",
            QuantityUnit::MilliLiter => "ml",
            QuantityUnit::Dash => "dash",
            QuantityUnit::Unit => "unit",
            QuantityUnit::Ounces => "oz",
            QuantityUnit::Drops => "drop",
            QuantityUnit::TableSpoon => "tbsp",
            QuantityUnit::TeaSpoon => "tsp",
            QuantityUnit::Cups => "cup",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for QuantityUnit {
    type Error = DataDomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "g" => Ok(QuantityUnit::Grams),
            "ml" => Ok(QuantityUnit::MilliLiter),
            "dash" => Ok(QuantityUnit::Dash),
            "unit" => Ok(QuantityUnit::Unit),
            "oz" => Ok(QuantityUnit::Ounces),
            "drop" => Ok(QuantityUnit::Drops),
            "tbsp" => Ok(QuantityUnit::TableSpoon),
            "tsp" => Ok(QuantityUnit::TeaSpoon),
            "cup" => Ok(QuantityUnit::Cups),
            _ => Err(DataDomainError::InvalidData),
        }
    }
}

impl TryFrom<&str> for RecipeCategory {
    type Error = DataDomainError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_ascii_lowercase();

        match value.as_str() {
            "easy" => Ok(RecipeCategory::Easy),
            "medium" => Ok(RecipeCategory::Medium),
            "advanced" => Ok(RecipeCategory::Advanced),
            "pro" => Ok(RecipeCategory::Pro),
            _ => Err(DataDomainError::InvalidRecipeCategory),
        }
    }
}

impl From<RecipeCategory> for String {
    fn from(val: RecipeCategory) -> Self {
        match val {
            RecipeCategory::Easy => "easy".into(),
            RecipeCategory::Medium => "medium".into(),
            RecipeCategory::Advanced => "advanced".into(),
            RecipeCategory::Pro => "pro".into(),
        }
    }
}

impl fmt::Display for RecipeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ss: String = match self {
            RecipeCategory::Easy => "easy".into(),
            RecipeCategory::Medium => "medium".into(),
            RecipeCategory::Advanced => "advanced".into(),
            RecipeCategory::Pro => "pro".into(),
        };

        write!(f, "{ss}")
    }
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
        id: Option<Uuid>,
        name: &str,
        image_id: Option<&str>,
        author_tags: Option<&[Tag]>,
        tags: Option<&[Tag]>,
        category: &str,
        description: Option<&str>,
        url: Option<&str>,
        ingredients: &[RecipeContains],
        steps: &[&str],
        author_id: Option<&str>,
    ) -> Result<Self, DataDomainError> {
        let category: RecipeCategory = category.try_into()?;

        tracing::info!("Author id: {:?}", author_id);

        let recipe = Recipe {
            id,
            name: name.into(),
            image_id: image_id.map(String::from),
            author_tags: author_tags.map(Vec::from),
            tags: tags.map(Vec::from),
            category,
            rating: Some(StarRate::Null),
            description: description.map(String::from),
            url: url.map(String::from),
            ingredients: Vec::from(ingredients),
            steps: steps.iter().map(|c| String::from(*c)).collect(),
            author_id: if let Some(id) = author_id {
                Some(Uuid::from_str(id).map_err(|_| {
                    error!("Wrong string given as Author ID: {id}");
                    DataDomainError::InvalidId
                })?)
            } else {
                None
            },
            creation_date: Some(Local::now()),
            update_date: None,
        };

        recipe.validate().map_err(|e| {
            error!("{e}");
            DataDomainError::InvalidFormData
        })?;

        Ok(recipe)
    }

    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn image_id(&self) -> Option<&str> {
        self.image_id.as_deref()
    }

    pub fn author_tags(&self) -> Option<&[Tag]> {
        self.author_tags.as_deref()
    }

    pub fn tags(&self) -> Option<&[Tag]> {
        self.tags.as_deref()
    }

    pub fn category(&self) -> RecipeCategory {
        self.category.clone()
    }

    pub fn rating(&self) -> StarRate {
        match &self.rating {
            Some(rating) => rating.clone(),
            None => StarRate::Null,
        }
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn ingredients(&self) -> &[RecipeContains] {
        &self.ingredients
    }

    pub fn steps(&self) -> &[String] {
        &self.steps
    }

    pub fn creation_date(&self) -> Option<DateTime<Local>> {
        self.creation_date
    }

    pub fn update_date(&self) -> Option<DateTime<Local>> {
        self.update_date
    }

    pub fn owner(&self) -> Option<Uuid> {
        self.author_id
    }
}

impl std::fmt::Display for RecipeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ss = String::new();

        if self.name.is_some() {
            ss.insert_str(ss.len(), &format!("name={} ", self.name.as_ref().unwrap()));
        }

        if self.tags.is_some() {
            ss.insert_str(ss.len(), &format!("tag={} ", self.tags.as_ref().unwrap()));
        }

        if self.rating.is_some() {
            ss.insert_str(
                ss.len(),
                &format!("rating={} ", self.rating.as_ref().unwrap()),
            );
        }

        if self.category.is_some() {
            let category = self.category.as_ref().unwrap();
            ss.insert_str(ss.len(), &format!("category={category} "));
        }

        write!(f, "Search tokens: {}", ss.strip_suffix(" ").unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use uuid::Uuid;

    struct TemplateRecipe<'a> {
        pub id: Uuid,
        pub name: String,
        pub image_id: Option<String>,
        pub author_tags: Option<Vec<Tag>>,
        pub tags: Option<Vec<Tag>>,
        pub category: String,
        pub description: Option<String>,
        pub url: Option<String>,
        pub ingredients: Vec<RecipeContains>,
        pub steps: &'a [&'a str],
        pub author_id: String,
    }

    #[fixture]
    fn template_recipe<'a>() -> TemplateRecipe<'a> {
        TemplateRecipe {
            id: Uuid::now_v7(),
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
            category: "easy".into(),
            description: Some("A delicious cocktail for summer.".to_owned()),
            url: None,
            ingredients: Vec::from([
                RecipeContains {
                    quantity: 100.0,
                    unit: QuantityUnit::Grams,
                    ingredient_id: Uuid::now_v7(),
                },
                RecipeContains {
                    quantity: 20.0,
                    unit: QuantityUnit::MilliLiter,
                    ingredient_id: Uuid::now_v7(),
                },
            ]),
            steps: &["Pour all the ingredients in a shaker", "Shake and serve"],
            author_id: Uuid::now_v7().to_string(),
        }
    }

    #[rstest]
    fn check_recipe_builds_using_valid_data(template_recipe: TemplateRecipe) {
        let recipe = Recipe::new(
            Some(template_recipe.id),
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            Some(&template_recipe.author_id.to_string()),
        );

        assert!(recipe.is_ok());

        let recipe = recipe.unwrap();

        assert_eq!(recipe.id.unwrap(), template_recipe.id);
        assert_eq!(recipe.name, template_recipe.name);
        assert_eq!(
            recipe.image_id.as_deref(),
            template_recipe.image_id.as_deref()
        );
        assert_eq!(recipe.author_tags, template_recipe.author_tags);
        assert_eq!(recipe.tags, template_recipe.tags);
        assert_eq!(
            recipe.category.to_string(),
            template_recipe.category.to_string()
        );
        assert_eq!(recipe.rating.unwrap(), StarRate::Null);
        assert_eq!(
            recipe.description.as_deref(),
            template_recipe.description.as_deref()
        );
        assert_eq!(recipe.url, template_recipe.url);
        assert_eq!(recipe.ingredients, template_recipe.ingredients);
        assert_eq!(recipe.steps, template_recipe.steps);
        assert_eq!(recipe.update_date, None);
        assert_eq!(
            recipe.author_id.unwrap().to_string(),
            template_recipe.author_id
        );
    }

    #[rstest]
    fn check_recipe_not_builds_using_invalid_data(template_recipe: TemplateRecipe) {
        // Invalid name test case
        let recipe = Recipe::new(
            Some(template_recipe.id),
            "Very long name that should produce an error",
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            Some(&template_recipe.author_id.to_string()),
        );

        assert!(recipe.is_err());

        // Invalid description test case
        let recipe = Recipe::new(
            Some(template_recipe.id),
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category,
            Some(&"An extremely long description".repeat(1000)),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            Some(&template_recipe.author_id.to_string()),
        );

        assert!(recipe.is_err());
    }

    #[rstest]
    fn check_recipe_getters(template_recipe: TemplateRecipe) {
        let recipe = Recipe::new(
            Some(template_recipe.id),
            &template_recipe.name,
            template_recipe.image_id.as_deref(),
            template_recipe.author_tags.as_deref(),
            template_recipe.tags.as_deref(),
            &template_recipe.category,
            template_recipe.description.as_deref(),
            template_recipe.url.as_deref(),
            &template_recipe.ingredients,
            template_recipe.steps,
            Some(&template_recipe.author_id.to_string()),
        );

        assert!(recipe.is_ok());

        let recipe = recipe.unwrap();

        assert_eq!(recipe.id().unwrap(), template_recipe.id);
        assert_eq!(recipe.name(), template_recipe.name);
        assert_eq!(
            recipe.image_id().as_deref(),
            template_recipe.image_id.as_deref()
        );
        assert_eq!(recipe.author_tags(), template_recipe.author_tags.as_deref());
        assert_eq!(recipe.tags(), template_recipe.tags.as_deref());
        assert_eq!(
            recipe.category().to_string(),
            template_recipe.category.to_string()
        );
        assert_eq!(recipe.rating(), StarRate::Null);
        assert_eq!(
            recipe.description().as_deref(),
            template_recipe.description.as_deref()
        );
        assert_eq!(recipe.url(), template_recipe.url.as_deref());
        assert_eq!(recipe.ingredients(), template_recipe.ingredients);
        assert_eq!(recipe.steps(), template_recipe.steps);
        assert_eq!(recipe.update_date(), None);
        assert_eq!(
            recipe.owner().unwrap().to_string(),
            template_recipe.author_id
        );
    }

    #[rstest]
    #[case("Easy", RecipeCategory::Easy)]
    #[case("mEdiUm", RecipeCategory::Medium)]
    #[case("PRO", RecipeCategory::Pro)]
    #[case("advanced", RecipeCategory::Advanced)]
    fn string_converts_to_recipe_category(#[case] input: &str, #[case] output: RecipeCategory) {
        let category = RecipeCategory::try_from(input).unwrap();
        assert_eq!(category, output);
    }

    #[rstest]
    #[case("easi")]
    #[case("adv")]
    fn wrong_string_fails_to_convert_to_recipe_category(#[case] input: &str) {
        match RecipeCategory::try_from(input) {
            Ok(_) => panic!("Conversion succeed when it should fail."),
            Err(e) => match e {
                DataDomainError::InvalidRecipeCategory => return,
                _ => panic!("Different type of error received"),
            },
        }
    }

    #[rstest]
    #[case(RecipeCategory::Easy, "easy")]
    #[case(RecipeCategory::Medium, "medium")]
    #[case(RecipeCategory::Advanced, "advanced")]
    #[case(RecipeCategory::Pro, "pro")]
    fn recipe_category_converts_to_string(#[case] category: RecipeCategory, #[case] value: &str) {
        let category: String = category.into();
        assert_eq!(&category, value);
    }

    #[rstest]
    #[case(StarRate::Null, "0")]
    #[case(StarRate::One, "1")]
    #[case(StarRate::Two, "2")]
    #[case(StarRate::Three, "3")]
    #[case(StarRate::Four, "4")]
    #[case(StarRate::Five, "5")]
    fn rating_converts_to_string(#[case] rating: StarRate, #[case] value: &str) {
        let category: String = format!("{rating}");
        assert_eq!(&category, value);
    }

    #[rstest]
    fn recipe_query_format() {
        let name = Some("Margarita".to_owned());
        let tags = None;
        let rating = None;
        let category = Some(RecipeCategory::Medium);
        let test_string = RecipeQuery {
            name: name.clone(),
            tags,
            rating,
            category: category.clone(),
        };
        let formatted_string = format!(
            "Search tokens: name={} category={}",
            name.unwrap(),
            category.unwrap()
        );
        let test_format = format!("{test_string}");
        assert_eq!(test_format, formatted_string);

        let name = None;
        let tags = Some("mocktail".to_owned());
        let rating = Some(StarRate::Null);
        let category = None;
        let test_string = RecipeQuery {
            name,
            tags: tags.clone(),
            rating: rating.clone(),
            category,
        };
        let formatted_string = format!(
            "Search tokens: tag={} rating={}",
            tags.unwrap(),
            rating.unwrap()
        );
        let test_format = format!("{test_string}");
        assert_eq!(test_format, formatted_string);
    }
}
