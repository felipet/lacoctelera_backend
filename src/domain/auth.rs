//! Data objects related to the authentication logic.

use crate::{domain::ID_LENGTH, DataDomainError};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::IntoParams;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, Validate, IntoParams)]
pub struct TokenRequestData {
    name: Option<String>,
    #[validate(email)]
    email: String,
    #[validate(length(min = 20, max = 400))]
    explanation: String,
}

impl TokenRequestData {
    pub fn new(
        name: Option<&str>,
        email: &str,
        explanation: &str,
    ) -> Result<Self, DataDomainError> {
        let data = TokenRequestData {
            name: name.map(|name| name.to_owned()),
            email: email.into(),
            explanation: explanation.into(),
        };

        match data.validate() {
            Ok(_) => Ok(data),
            Err(_) => Err(DataDomainError::InvalidFormData),
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

impl fmt::Display for TokenRequestData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Request explanation: \"{}\" by {}",
            self.explanation, self.email
        )
    }
}

/// Simple type to represent IDs for the API clients.
#[derive(Clone, Debug, Deserialize)]
pub struct ClientId(String);

impl ClientId {
    pub fn new() -> Self {
        let mut id = Uuid::now_v7().to_string();
        id.truncate(ID_LENGTH);

        Self(id)
    }
}

impl FromStr for ClientId {
    type Err = DataDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != ID_LENGTH {
            Err(DataDomainError::InvalidId)
        } else {
            Ok(ClientId(s.to_string()))
        }
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
