// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use actix_web::{options, HttpRequest, HttpResponse};

#[utoipa::path(
    options,
    path = "/ingredient",
    tag = "Ingredient",
    responses(
        (
            status = 204,
            description = "Supported requests to the /ingredient endpoint",
            headers(
                ("access-control-allow-headers", description = "content-type"),
                ("access-control-allow-origin", description = "*"),
                ("access-control-allow-methods", description = "GET, POST, OPTIONS")
            )
        )
    )
)]
#[options("/ingredient")]
pub async fn options_ingredient(_req: HttpRequest) -> HttpResponse {
    HttpResponse::NoContent()
        .append_header(("access-control-allow-headers", "content-type"))
        .append_header(("access-control-allow-origin", "*"))
        .append_header(("access-control-allow-methods", "GET, POST, OPTIONS"))
        .finish()
}
