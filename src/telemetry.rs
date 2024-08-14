use crate::configuration::LogSettings;
use std::fs::OpenOptions;
use tracing_subscriber::{fmt, prelude::*, Layer};

pub fn configure_tracing(conf: &LogSettings) {
    // Store all the tracing layers in an array to allow a dynamic configuration
    // using the given settings to the app.
    let mut layers = Vec::new();

    // First layer: log to a file (default).
    let fname = conf.log_output_file.as_str();
    let log_file_out = OpenOptions::new()
        .append(true)
        .create(true)
        .open(fname)
        .unwrap_or_else(|_| panic!("Failed to create the output log file: '{fname}'"));

    let layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(log_file_out)
        .with_filter(conf.get_verbosity_level())
        .boxed();
    layers.push(layer);

    // Optional layer: console output.
    if conf.console_log_enabled() {
        let layer = fmt::layer()
            .with_target(false)
            .pretty()
            .with_filter(conf.get_console_tracing_level())
            .boxed();

        layers.push(layer);
    }

    tracing_subscriber::registry().with(layers).init();
}
