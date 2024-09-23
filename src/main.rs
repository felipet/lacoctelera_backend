// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
