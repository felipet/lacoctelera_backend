use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use serde::Serialize;

#[allow(dead_code)]
#[derive(Serialize)]
enum Category {
    Spirit,
    Bitter,
    SoftDrink,
    Garnish,
    Other,
}
#[derive(Serialize)]
struct Ingredient {
    name: String,
    category: Category,
    desc: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world from La Coctelera!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/ingredient")]
async fn ingredient() -> Result<impl Responder> {
    let test_ingredient = Ingredient {
        name: "Vodka".to_string(),
        category: Category::Spirit,
        desc: "Voka liquour 40%".to_string(),
    };

    Ok(web::Json(test_ingredient))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(echo).service(ingredient))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
