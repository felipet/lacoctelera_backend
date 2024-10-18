//! Data objects related to the authentication logic.

use core::fmt;

use crate::DataDomainError;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
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
