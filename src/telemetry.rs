// Copyright 2024-2025 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::configuration::LogSettings;
use tracing::error;
use tracing_subscriber::{fmt, fmt::Formatter, prelude::*, Layer};

pub fn configure_tracing(conf: &LogSettings) {
    // Store all the tracing layers in an array to allow a dynamic configuration
    // using the given settings to the app.
    let mut layers = Vec::new();

    let tracing_levelfilter = conf.get_verbosity_level();

    if conf.journald.unwrap_or_default() {
        match tracing_journald::layer() {
            Ok(layer) => {
                layers.push(
                    layer
                        .with_field_prefix(Some("lacoctelera_backend".to_owned()))
                        .with_filter(tracing_levelfilter)
                        .boxed(),
                );
            }
            // journald is typically available on Linux systems, but nowhere else. Portable software
            // should handle its absence gracefully.
            Err(e) => {
                error!("couldn't connect to journald: {e}");
            }
        }
    } else if conf.pretty_log.unwrap_or_default() {
        // Configure the default layer: STDOUT when not running as a systemd service
        let layer = fmt::layer()
            .pretty()
            .with_target(false)
            .with_filter(tracing_levelfilter)
            .boxed();

        layers.push(layer);
    } else {
        // Configure the default layer: STDOUT when not running as a systemd service
        let layer = fmt::layer()
            .without_time()
            .with_ansi(false)
            .with_target(false)
            .with_filter(tracing_levelfilter)
            .boxed();

        layers.push(layer);
    }

    tracing_subscriber::registry().with(layers).init();
}
