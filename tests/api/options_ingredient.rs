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
}
