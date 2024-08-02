use lacoctelera::{configuration::Settings, startup::Application};

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    let configuration = Settings::new().expect("Failed to parse configuration files.");

    let app = Application::build(configuration).await?;
    app.run_until_stopped().await?;

    Ok(())
}
