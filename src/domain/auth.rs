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
        let full_id = Uuid::now_v7().to_string();
        let first_chunk = &full_id[..ID_LENGTH / 2];
        let second_chunk = &full_id[full_id.len() - ID_LENGTH / 2..];

        Self(String::from(&format!("{first_chunk}{second_chunk}")))
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use rstest::*;

    #[rstest]
    fn valid_data_builds() {
        let name = "John Doe";
        let email = "john_doe@mail.com";
        let explanation = "I need access to destroy your DB";

        let valid_token_request = TokenRequestData::new(Some(name), email, explanation);

        assert!(valid_token_request.is_ok());

        let valid = valid_token_request.unwrap();
        assert_eq!(valid.name(), Some(name));
        assert_eq!(valid.email(), email);
        assert_eq!(valid.explanation(), explanation);

        let name = None;

        let valid_token_request = TokenRequestData::new(name, email, explanation);

        assert!(valid_token_request.is_ok());

        let valid = valid_token_request.unwrap();
        assert_eq!(valid.name(), name);
        assert_eq!(valid.email(), email);
        assert_eq!(valid.explanation(), explanation);

        assert_eq!(
            &format!("{valid}"),
            "Request explanation: \"I need access to destroy your DB\" by john_doe@mail.com"
        );
    }

    #[rstest]
    fn wrong_data_fails_to_build() {
        let email = "john doe";
        let explanation = "I need access to destroy your DB";

        let valid_token_request = TokenRequestData::new(None, email, explanation);

        assert!(valid_token_request.is_err());

        let email = "johndoe@mail.com";
        let explanation = "Give me";

        let valid_token_request = TokenRequestData::new(None, email, explanation);

        assert!(valid_token_request.is_err());
    }

    #[rstest]
    fn construct_new_client_id() {
        let client_id1 = ClientId::default();
        let client_id2 = ClientId::default();

        assert_ne!(client_id1.0, client_id2.0);
        assert!(client_id1.0.to_string().len() == ID_LENGTH);
        assert!(ClientId::from_str("0399ab0f").is_ok());
        assert!(ClientId::from_str("0399ab0ñ").is_err());
        assert!(ClientId::from_str("0399ab0f92").is_err());

        assert_eq!(format!("{}", client_id1.0), format!("{client_id1}"));
    }
}
