// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::domain::Recipe;
use sqlx::MySqlPool;
use std::error::Error;
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(_pool))]
pub async fn register_new_recipe(
    _pool: &MySqlPool,
    _recipe: &Recipe,
) -> Result<Uuid, Box<dyn Error>> {
    todo!()
}
