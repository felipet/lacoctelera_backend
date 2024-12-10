// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use lacoctelera::domain::{Author, AuthorBuilder, SocialProfile};
use sqlx::{Executor, MySqlPool};
use std::fs;
use tracing::error;
use uuid::Uuid;

pub struct FixtureSeeder<'a> {
    db_pool: &'a MySqlPool,
    seed_authors: Option<bool>,
    seed_social_profiles: Option<bool>,
}

impl<'a> FixtureSeeder<'a> {
    fn new(db_pool: &'a MySqlPool) -> Self {
        FixtureSeeder {
            db_pool,
            seed_authors: None,
            seed_social_profiles: None,
        }
    }

    pub fn with_authors(mut self) -> FixtureSeeder<'a> {
        self.seed_authors = Some(true);

        self
    }

    pub fn with_social_profiles(mut self) -> FixtureSeeder<'a> {
        self.seed_social_profiles = Some(true);

        self
    }

    pub async fn seed(&self) -> Result<FixtureMap, String> {
        let mut map = FixtureMap::default();

        if self.seed_social_profiles.is_some() {
            let mut social_profile_fixture = SocialProfileFixture::default();
            social_profile_fixture.load()?;
            social_profile_fixture.seed(self.db_pool).await?;
            map.social = Some(social_profile_fixture);
        }

        if self.seed_authors.is_some() {
            let mut author_fixture = AuthorFixture::default();
            author_fixture.load()?;
            author_fixture
                .seed(self.db_pool, self.seed_social_profiles.unwrap_or_default())
                .await?;
        }

        Ok(map)
    }
}

#[derive(Default)]
pub struct FixtureMap {
    pub author: Option<AuthorFixture>,
    pub social: Option<SocialProfileFixture>,
}

#[derive(Default)]
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

#[derive(Default)]
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
