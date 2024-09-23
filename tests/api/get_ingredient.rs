// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::{spawn_app, TestApp};
use lacoctelera::{routes::ingredient::QueryData, Ingredient};
use sqlx::Executor;

struct TestFixtureDataBase {
    pub test_app: TestApp,
    pub test_ingredients: Vec<Ingredient>,
}

type FixtureResult = Result<TestFixtureDataBase, anyhow::Error>;

async fn seed_ingredients() -> FixtureResult {
    let test_app = spawn_app().await;

    let test_ingredients = Vec::from([
        Ingredient::parse("Vodka", "spirit", Some("Regular Vodka 40%")).unwrap(),
        Ingredient::parse("White Rum", "spirit", Some("Any white Rum")).unwrap(),
        Ingredient::parse("Lime Super Juice", "other", None).unwrap(),
        Ingredient::parse("Agave Sirup", "other", None).unwrap(),
        Ingredient::parse("Soda water", "soft_drink", None).unwrap(),
        Ingredient::parse(
            "Absolut Vodka",
            "spirit",
            Some("Only Absolut gives the needed flavor profile."),
        )
        .unwrap(),
    ]);

    let mut conn = test_app.db_pool.acquire().await.unwrap();

    for ingredient in test_ingredients.iter() {
        let query = sqlx::query!(
            r#"
            INSERT INTO Ingredient (name, category, `desc`) VALUES
                (?,?,?)
            "#,
            ingredient.name(),
            ingredient.category().to_string(),
            ingredient.desc(),
        );

        conn.execute(query)
            .await
            .expect("Failed to seed ingredients in the DB");
    }

    conn.close().await?;

    Ok(TestFixtureDataBase {
        test_app,
        test_ingredients,
    })
}

#[actix_web::test]
async fn get_ingredient_returns_200_for_valid_query() {
    let test_app = spawn_app().await;

    let test_queries = [("vodka", None), ("coconut liquour", None)];

    for query in test_queries.iter() {
        println!("Testing: {:#?}", query);
        let response = test_app
            .get_ingredient(
                &QueryData {
                    name: query.0.to_owned(),
                },
                query.1,
            )
            .await;

        assert_eq!(200, response.status().as_u16());
    }
}

#[actix_web::test]
async fn get_ingredient_returns_400_for_invalid_query() {
    let test_app = spawn_app().await;

    let test_queries = [("vod;ka", "name"), ("vodka", "ingredient"), ("vodka", "")];

    for query in test_queries.iter() {
        println!("Testing: {:#?}", query);
        let response = test_app
            .get_ingredient(
                &QueryData {
                    name: query.0.to_owned(),
                },
                Some(query.1),
            )
            .await;

        assert_eq!(400, response.status().as_u16());
    }
}

#[actix_web::test]
async fn get_existing_ingredient() {
    let test_fixture = seed_ingredients()
        .await
        .expect("Failed to seed ingredients into the test DB.");

    for ingredient in test_fixture.test_ingredients.iter() {
        let response = test_fixture
            .test_app
            .get_ingredient(
                &QueryData {
                    name: ingredient.name().to_owned(),
                },
                None,
            )
            .await;

        let ingredients: Vec<Ingredient> = response.json().await.unwrap();

        assert!(ingredients.contains(ingredient));
    }
}
