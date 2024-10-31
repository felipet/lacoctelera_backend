// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::helpers::spawn_app;
use reqwest::{Method, Request, Url};

#[actix_web::test]
async fn options_ingredient_returns_204() {
    let test_app = spawn_app().await;

    let request = Request::new(
        Method::OPTIONS,
        Url::parse(&format!("{}/ingredient", test_app.address.as_str())).unwrap(),
    );

    let response = test_app
        .api_client
        .execute(request)
        .await
        .expect("Error in OPTIONS request");

    assert_eq!(response.status().as_u16(), 204);

    test_app.db_pool.close().await;
}

#[actix_web::test]
async fn options_ingredient_returns_expected_headers() {
    let test_app = spawn_app().await;

    let request = Request::new(
        Method::OPTIONS,
        Url::parse(&format!("{}/ingredient", test_app.address.as_str())).unwrap(),
    );

    let response = test_app
        .api_client
        .execute(request)
        .await
        .expect("Error in OPTIONS request");

    assert!(response
        .headers()
        .contains_key("access-control-allow-headers"));
    assert!(response
        .headers()
        .contains_key("access-control-allow-origin"));

    test_app.db_pool.close().await;
}
