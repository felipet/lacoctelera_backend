//! Data objects related to Authors.

use names::Generator;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

static RE_UUID_V4: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-fA-F0-9-]{4,12}){5}$").unwrap());

/// Object that represents an Author of the `Cocktail` data base.
///
/// # Description
///
/// Authors' main role is defining recipes in the data base, and own them until they delete them or transfer the
/// ownership. This object is a simple descriptor that includes some personal information of the authors.
///
/// Most of the attributes are optional, and authors are given the choice to share or keep private their profiles.
/// When a profile is set as private, only the [Author::name] is shown while listing a recipe.
///
/// The constructor [Author::default] is given to generate a new author entry using a random funny name.
///
/// All the fields that accept a text input are checked to avoid exceeding the maximum allowed text length.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct Author {
    #[validate(custom(function = "validate_id"))]
    #[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    id: Uuid,
    #[validate(length(min = 1), length(max = 40))]
    name: String,
    #[validate(length(min = 1), length(max = 40))]
    surname: Option<String>,
    #[validate(email)]
    email: Option<String>,
    /// Decide whether an author profile can be shared to the public or not.
    pub shareable: bool,
    #[validate(length(max = 255))]
    description: Option<String>,
    #[validate(url)]
    website: Option<String>,
    social_profiles: Option<Vec<SocialProfile>>,
}

/// Custom function to validate a String that should contain an [Uuid].
fn validate_id(value: &Uuid) -> Result<(), ValidationError> {
    if RE_UUID_V4.is_match(&value.to_string()) {
        std::result::Result::Ok(())
    } else {
        Err(ValidationError::new("1"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SocialProfile {
    pub id: u16,
    pub provider_name: String,
    pub user_name: String,
}

#[derive(Error, Debug)]
pub enum AuthorError {
    #[error("Some params contain an invalid format.")]
    InvalidParams {
        #[from]
        source: ValidationErrors,
    },
    #[error("The given Author ID hash an invalid format.")]
    InvalidId,
}

impl std::default::Default for Author {
    fn default() -> Self {
        Author {
            id: Uuid::now_v7(),
            name: Generator::default().next().unwrap(),
            surname: None,
            email: None,
            shareable: false,
            description: None,
            website: None,
            social_profiles: None,
        }
    }
}

impl Author {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        surname: Option<String>,
        email: Option<String>,
        shareable: bool,
        description: Option<String>,
        website: Option<String>,
        social_profiles: Option<Vec<SocialProfile>>,
    ) -> Result<Self, AuthorError> {
        let id = match Uuid::from_str(&id) {
            Ok(id) => id,
            Err(_) => return Err(AuthorError::InvalidId),
        };

        let author = Author {
            id,
            name,
            surname,
            email,
            shareable,
            description,
            website,
            social_profiles,
        };

        match author.validate() {
            Ok(_) => std::result::Result::Ok(author),
            Err(e) => Err(AuthorError::InvalidParams { source: e }),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn surname(&self) -> Option<&str> {
        self.surname.as_deref()
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn shareable(&self) -> bool {
        self.shareable
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn website(&self) -> Option<&str> {
        self.website.as_deref()
    }

    pub fn social_profiles(&self) -> Option<&[SocialProfile]> {
        self.social_profiles.as_deref()
    }
}

#[derive(Default)]
pub struct AuthorBuilder {
    id: Option<String>,
    name: Option<String>,
    surname: Option<String>,
    email: Option<String>,
    shareable: bool,
    description: Option<String>,
    website: Option<String>,
    social_profiles: Option<Vec<SocialProfile>>,
}

impl AuthorBuilder {
    pub fn set_id(mut self, id: &str) -> Self {
        self.id = Some(id.into());

        self
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.into());

        self
    }

    pub fn set_surname(mut self, surname: &str) -> Self {
        self.surname = Some(surname.into());

        self
    }

    pub fn set_email(mut self, email: &str) -> Self {
        self.email = Some(email.into());

        self
    }

    pub fn set_shareable(mut self, shareable: bool) -> Self {
        self.shareable = shareable;

        self
    }

    pub fn set_description(mut self, description: &str) -> Self {
        self.description = Some(description.into());

        self
    }

    pub fn set_website(mut self, website: &str) -> Self {
        self.website = Some(website.into());

        self
    }

    pub fn set_social_profiles(mut self, profiles: &[SocialProfile]) -> Self {
        self.social_profiles = Some(Vec::from(profiles));

        self
    }

    pub fn build(self) -> Result<Author, AuthorError> {
        let id = match self.id.as_ref() {
            Some(id) => id.clone(),
            None => Uuid::now_v7().to_string(),
        };

        let name = match self.name.as_ref() {
            Some(name) => name.clone(),
            None => Generator::default().next().unwrap(),
        };

        Author::new(
            id,
            name,
            self.surname,
            self.email,
            self.shareable,
            self.description,
            self.website,
            self.social_profiles,
        )
    }
}
