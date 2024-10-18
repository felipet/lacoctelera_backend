use crate::helpers::spawn_app;

#[actix_web::test]
async fn request_returns_202_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = serde_json::json!({
        "email": "janedoe@mail.com",
        "explanation": "A_very_long_sentence_for_testing",
    });

    let response = test_app.post_token_request(&body).await;

    assert_eq!(202, response.status().as_u16());
}

#[actix_web::test]
async fn request_returns_200_for_existing_email() {
    let test_app = spawn_app().await;

    let body = serde_json::json!({
        "email": "janedoe@mail.com",
        "explanation": "A_very_long_sentence_for_testing",
    });

    let response = test_app.post_token_request(&body).await;

    // The first time, it shall return Ok (202).
    assert_eq!(202, response.status().as_u16());

    // Attempt to register twice the same email.
    let response = test_app.post_token_request(&body).await;

    // This time, the response shall be Ok (200).
    assert_eq!(200, response.status().as_u16());
}
