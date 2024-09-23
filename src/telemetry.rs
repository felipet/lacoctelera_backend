// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
            .pretty()
            // Disable some options enabled by pretty that are not useful
            .with_target(false)
            .with_file(false)
            .with_line_number(false)
            .with_filter(conf.get_console_tracing_level())
            .boxed();

        layers.push(layer);
    }

    tracing_subscriber::registry().with(layers).init();
}
