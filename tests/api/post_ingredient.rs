// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::spawn_app;
use lacoctelera::{routes::ingredient::FormData, IngCategory, Ingredient};
use sqlx::query_as;
use uuid::Uuid;

#[actix_web::test]
async fn post_ingredient_returns_200_for_valid_json() {
    let test_app = spawn_app().await;

    let test_payload = [
        (
            FormData {
                name: "tc1".to_string(),
                category: IngCategory::Spirit.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Spirit test case",
        ),
        (
            FormData {
                name: "tc2".to_string(),
                category: IngCategory::Bitter.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Bitter test case",
        ),
        (
            FormData {
                name: "tc3".to_string(),
                category: IngCategory::Garnish.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Garnish test case",
        ),
        (
            FormData {
                name: "tc4".to_string(),
                category: IngCategory::SoftDrink.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "SoftDrink test case",
        ),
        (
            FormData {
                name: "tc5".to_string(),
                category: IngCategory::Other.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Other test case",
        ),
        (
            FormData {
                name: "My drink 80%".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "Composed name test case",
        ),
        (
            FormData {
                name: "tc7".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "No description teste case",
        ),
    ];

    for (payload, err_msg) in test_payload.iter() {
        println!("{err_msg}");
        println!("Ingredient POST using: {:#?}", payload);
        let response = test_app.post_ingredient(&payload).await;
        assert_eq!(response.status().as_u16(), 200);
    }

    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn post_ingredient_returns_400_for_invalid_json() {
    let test_app = spawn_app().await;

    let test_payload = [
        (
            FormData {
                name: "1nvalid".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "Wrong name format test case 1",
        ),
        (
            FormData {
                name: "alco;hol".to_string(),
                category: IngCategory::Other.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Wrong name format test case 2",
        ),
        (
            FormData {
                name: "tc3".to_string(),
                category: "my invented category".to_string(),
                desc: None,
            },
            "Non existing category test case",
        ),
    ];

    for (payload, err_msg) in test_payload.iter() {
        println!("{err_msg}");
        println!("Ingredient POST using: {:#?}", payload);
        let response = test_app.post_ingredient(&payload).await;
        assert_eq!(response.status().as_u16(), 400);
    }

    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn post_ingredient_to_db() {
    let test_app = spawn_app().await;

    let test_payload = [
        (
            FormData {
                name: "tc1".to_string(),
                category: IngCategory::Spirit.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Spirit test case",
        ),
        (
            FormData {
                name: "tc2".to_string(),
                category: IngCategory::Bitter.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Bitter test case",
        ),
        (
            FormData {
                name: "tc3".to_string(),
                category: IngCategory::Garnish.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Garnish test case",
        ),
        (
            FormData {
                name: "tc4".to_string(),
                category: IngCategory::SoftDrink.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "SoftDrink test case",
        ),
        (
            FormData {
                name: "tc5".to_string(),
                category: IngCategory::Other.to_string(),
                desc: Some(Uuid::new_v4().to_string()),
            },
            "Other test case",
        ),
        (
            FormData {
                name: "My drink 80%".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "Composed name test case",
        ),
        (
            FormData {
                name: "tc7".to_string(),
                category: IngCategory::Other.to_string(),
                desc: None,
            },
            "No description teste case",
        ),
    ];

    let _ = actix_web::rt::time::sleep(std::time::Duration::from_secs(60));

    for (payload, err_msg) in test_payload.iter() {
        println!("{err_msg}");
        println!("Ingredient POST using: {:#?}", payload);
        let response = test_app.post_ingredient(&payload).await;

        println!("{:#?}", response);

        let _ = actix_web::rt::time::sleep(std::time::Duration::from_secs(60));

        let query = query_as!(
            FormData,
            r#"SELECT name, category, `desc`
            FROM Ingredient
            WHERE name = ?"#,
            payload.name,
        )
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to query DB");

        assert_eq!(
            Ingredient::parse(&payload.name, &payload.category, payload.desc.as_deref()).unwrap(),
            Ingredient::parse(&query.name, &query.category, query.desc.as_deref()).unwrap(),
        );
    }

    test_app.db_pool.close().await;
}
