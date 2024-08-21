use lacoctelera::{configuration::Settings, startup::Application, telemetry::configure_tracing};
use tracing::{debug, info};

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    let configuration = Settings::new().expect("Failed to parse configuration files.");

    // Set up the tracing sub-system.
    configure_tracing(&configuration.application.log_settings);

    info!(
        "La Coctelera API started @ {}",
        configuration.application.port
    );

    let app = Application::build(configuration).await?;
    debug!("Application built, serving requests");
    app.run_until_stopped().await?;

    Ok(())
}
