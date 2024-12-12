// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use adv_random;
use lacoctelera::{
    domain::{Author, AuthorBuilder, QuantityUnit, Recipe, RecipeContains, SocialProfile},
    Ingredient,
};
use sqlx::{Executor, MySqlPool};
use std::fs;
use tracing::error;
use uuid::Uuid;

pub struct FixtureSeeder<'a> {
    db_pool: &'a MySqlPool,
    seed_authors: Option<bool>,
    seed_social_profiles: Option<bool>,
    seed_ingredients: Option<bool>,
    seed_recipes: Option<bool>,
}

impl<'a> FixtureSeeder<'a> {
    pub fn new(db_pool: &'a MySqlPool) -> Self {
        FixtureSeeder {
            db_pool,
            seed_authors: None,
            seed_social_profiles: None,
            seed_ingredients: None,
            seed_recipes: None,
        }
    }

    pub fn with_authors(mut self, seed: bool) -> FixtureSeeder<'a> {
        self.seed_authors = Some(seed);

        self
    }

    pub fn with_social_profiles(mut self, seed: bool) -> FixtureSeeder<'a> {
        self.seed_social_profiles = Some(seed);

        self
    }

    pub fn with_ingredients(mut self, seed: bool) -> FixtureSeeder<'a> {
        self.seed_ingredients = Some(seed);

        self
    }

    pub fn with_recipes(mut self, seed: bool) -> FixtureSeeder<'a> {
        self.seed_recipes = Some(seed);

        self
    }

    pub async fn seed(&self) -> Result<FixtureMap, String> {
        let mut map = FixtureMap::default();

        if self.seed_social_profiles.is_some() {
            let mut social_profile_fixture = SocialProfileFixture::default();
            social_profile_fixture.load()?;
            if self.seed_social_profiles.unwrap() {
                social_profile_fixture.seed(self.db_pool).await?;
            }
            map.social = Some(social_profile_fixture);
        }

        if self.seed_recipes.is_some() {
            let mut recipe_fixture = RecipeFixture::default();
            recipe_fixture.load()?;
            if self.seed_recipes.unwrap() {
                recipe_fixture.seed(self.db_pool).await?;
            }
            map.recipe = Some(recipe_fixture);
        } else {
            if self.seed_authors.is_some() {
                let mut author_fixture = AuthorFixture::default();
                author_fixture.load()?;
                if self.seed_authors.unwrap() {
                    author_fixture
                        .seed(self.db_pool, self.seed_social_profiles.unwrap_or_default())
                        .await?;
                }
                map.author = Some(author_fixture);
            }
            if self.seed_ingredients.is_some() {
                let mut ingredient_fixture = IngredientFixture::default();
                ingredient_fixture.load()?;
                if self.seed_ingredients.unwrap() {
                    ingredient_fixture.seed(self.db_pool).await?;
                }
                map.ingredient = Some(ingredient_fixture);
            }
        }

        Ok(map)
    }
}

#[derive(Debug, Default)]
pub struct FixtureMap {
    pub author: Option<AuthorFixture>,
    pub social: Option<SocialProfileFixture>,
    pub ingredient: Option<IngredientFixture>,
    pub recipe: Option<RecipeFixture>,
}

#[derive(Debug, Default)]
pub struct AuthorFixture {
    pub valid_fixtures: Vec<Author>,
}

impl AuthorFixture {
    pub fn load(&mut self) -> Result<(), String> {
        let file =
            fs::read_to_string("tests/api/fixtures/authors.yml").map_err(|e| e.to_string())?;
        self.valid_fixtures = serde_yml::from_str(&file).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn seed(
        &mut self,
        pool: &MySqlPool,
        use_social_profiles: bool,
    ) -> Result<(), String> {
        let mut ids = Vec::new();

        for author in self.valid_fixtures.iter() {
            let id = Uuid::now_v7();

            let mut transaction = pool.begin().await.map_err(|e| {
                error!("{e}");
                e.to_string()
            })?;

            transaction.execute(
            sqlx::query!(
                r#"INSERT INTO `Author`(`id`, `name`, `surname`, `email`, `shareable`, `description`, `website`)
                VALUES (?,?,?,?,?,?,?)"#,
                id.to_string(),
                author.name(),
                author.surname(),
                author.email(),
                author.shareable(),
                author.description(),
                author.website()
            )).await.map_err(|e| {error!("{e}"); e.to_string()})?;

            ids.push(id);

            if use_social_profiles {
                if let Some(profiles) = author.social_profiles() {
                    for profile in profiles {
                        transaction.execute(
                        sqlx::query!(
                            r#"INSERT INTO `AuthorHashSocialProfile`(`id`, `provider_name`, `user_name`, `author_id`)
                            VALUES (?,?,?,?)"#,
                            Uuid::now_v7().to_string(),
                            profile.provider_name,
                            profile.website,
                            id.to_string(),
                        )).await.map_err(|e| {error!("{e}"); e.to_string()})?;
                    }
                }
            }

            transaction.commit().await.map_err(|e| {
                error!("{e}");
                e.to_string()
            })?;
        }

        for it in 0..ids.len() {
            let author = AuthorBuilder::default()
                .set_id(&ids[it].to_string())
                .build()
                .expect("Wrong ID");
            self.valid_fixtures[it].update_from(&author);
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SocialProfileFixture {
    pub valid_fixtures: Vec<SocialProfile>,
}

impl SocialProfileFixture {
    pub fn load(&mut self) -> Result<(), String> {
        let file = fs::read_to_string("tests/api/fixtures/social_profiles.yml")
            .map_err(|e| e.to_string())?;
        self.valid_fixtures = serde_yml::from_str(&file).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn seed(&self, pool: &MySqlPool) -> Result<(), String> {
        for profile in self.valid_fixtures.iter() {
            sqlx::query!(
                "INSERT INTO `SocialProfile` VALUES (?, ?)",
                profile.provider_name,
                profile.website
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn valid_fixtures(&self) -> &[SocialProfile] {
        &self.valid_fixtures
    }
}

#[derive(Debug, Default)]
pub struct IngredientFixture {
    pub valid_fixtures: Vec<Ingredient>,
}

impl IngredientFixture {
    pub fn load(&mut self) -> Result<(), String> {
        let file =
            fs::read_to_string("tests/api/fixtures/ingredients.yml").map_err(|e| e.to_string())?;
        self.valid_fixtures = serde_yml::from_str(&file).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn seed(&mut self, pool: &MySqlPool) -> Result<(), String> {
        for ingredient in self.valid_fixtures.iter_mut() {
            ingredient.set_id(Uuid::now_v7());

            sqlx::query!(
                "INSERT INTO `Ingredient` VALUES (?,?,?,?)",
                ingredient.id().unwrap().to_string(),
                ingredient.name(),
                ingredient.category().to_str().to_owned(),
                ingredient.desc(),
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn valid_fixtures(&self) -> &[Ingredient] {
        &self.valid_fixtures
    }
}

#[derive(Debug, Default)]
pub struct RecipeFixture {
    pub valid_fixtures: Vec<Recipe>,
}

impl RecipeFixture {
    pub fn load(&mut self) -> Result<(), String> {
        let file =
            fs::read_to_string("tests/api/fixtures/recipes.yml").map_err(|e| e.to_string())?;
        self.valid_fixtures = serde_yml::from_str(&file).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn seed(&mut self, pool: &MySqlPool) -> Result<(), String> {
        // First, we need to seed, at least, an author and a few ingredients to build up a decent recipe.
        let mut ingredient_fixture = IngredientFixture::default();
        ingredient_fixture.load()?;
        ingredient_fixture.seed(pool).await?;

        let ingredients = ingredient_fixture.valid_fixtures;

        let mut author_fixture = AuthorFixture::default();
        author_fixture.load()?;
        author_fixture.seed(pool, false).await?;

        let authors = author_fixture.valid_fixtures;

        // The author and the ingredients will be selected randomly.
        let random_result =
            adv_random::random::random_numbers(&adv_random::settings::Settings::new(
                &[Box::new(adv_random::rules::NumberRange::all(
                    0,
                    ingredients.len() - 1,
                ))],
                3,
            ));
        let random_ingredients = match random_result.numbers() {
            Ok(numbers) => numbers,
            _ => return Err("Failed to generate random numbers".to_owned()),
        };

        // Now, let's indicate what ingredients will be used in the recipe.
        let included_ingredients = &[
            RecipeContains {
                quantity: 1.0,
                unit: QuantityUnit::Ounces,
                ingredient_id: ingredients[random_ingredients[0]].id().unwrap(),
            },
            RecipeContains {
                quantity: 30.0,
                unit: QuantityUnit::MilliLiter,
                ingredient_id: ingredients[random_ingredients[1]].id().unwrap(),
            },
        ];

        let recipe = Recipe::new(
            None,
            "Dummy Recipe",
            None,
            None,
            None,
            "easy",
            None,
            None,
            included_ingredients,
            &["Pour everything into a cup and enjoy."],
            authors[0].id().as_deref(),
        )
        .map_err(|e| e.to_string())?;

        self.valid_fixtures.push(recipe);

        Ok(())
    }
}
