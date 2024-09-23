// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Data objects related to Authors.

use crate::{domain::DataDomainError, validate_id};
use names::Generator;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Object that represents an Author of the `Cocktail` data base.
///
/// # Description
///
/// Authors' main role is defining recipes in the data base, and own them until they delete them or transfer the
/// ownership. This object is a simple descriptor that includes some personal information of the authors.
///
/// All attributes are optional to allow using this `struct` as query object for the GET methods of the `/author`
/// endpoint. Mandatory fields are defined in the description of each method of the endpoint.
///
/// Some restrictions over the `struct`'s members:
/// - [Author::id] must contain a valid [Uuid]. Strings are parsed to [Uuid]. The expected format is a 128-bit value,
///   formatted as a hex string in five groups. The first 4 groups are randomly generated, and the fifth comes from
///   a timestamp. However, clients can freely generate this ID using other combinations as long as the length and basic
///   format rules are honored.
/// - [Author::name] and [Author::surname] shall have a minimum length of 2 and a maximum of 40 characters. These
///   fields are allowed to repeat in the DB. Authors are identified in the DB by [Author::id]. Usernames are not
///   required.
/// - [Author::email] is validated against the HTML5 regex.
/// - [Author::description] can't exceed 255 characters length.
/// - [Author::website] must contain an url format (`http://...` or `https://...`).
///
/// Authors are given the choice to share or keep private their profiles. Activate [Author::shareable] to allow
/// sharing the author's profile to the main public. Private profiles are protected from non privileged clients of the
/// API (with no API access token): only the [Author::id] and [Author::name] is given when a unprivileged client
/// requests the data of an author to the API.
///
/// The constructor [Author::default] is given to generate a new author entry using a random funny name.
///
/// Prefer [AuthorBuilder] rather than [Author::new] to build a new [Author] instance.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, IntoParams, Validate, PartialEq)]
pub struct Author {
    #[validate(custom(function = "validate_id"))]
    #[schema(value_type = String, example = "0191e13b-5ab7-78f1-bc06-be503a6c111b")]
    id: Option<Uuid>,
    #[validate(length(min = 2), length(max = 40))]
    name: Option<String>,
    #[validate(length(min = 2), length(max = 40))]
    surname: Option<String>,
    #[validate(email)]
    email: Option<String>,
    /// Decide whether an author profile can be shared to the public or not.
    pub shareable: Option<bool>,
    #[validate(length(max = 255))]
    description: Option<String>,
    #[validate(url)]
    website: Option<String>,
    social_profiles: Option<Vec<SocialProfile>>,
}

/// Simple Data object to describe a social network profile.
///
/// # Description
///
/// [SocialProfile] has no associated table in the DB. This is simply contained in the [Author]'s entry in the DB.
/// A social profile is described by the social network name and the author's user name.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, Validate, PartialEq)]
pub struct SocialProfile {
    /// Name of the social network, i.e. Instagram, X, TikTok... 40 chars max.
    #[validate(length(max = 40))]
    pub provider_name: String,
    /// User name registered by the author in the social network. 40 chars max.
    #[validate(length(max = 40))]
    pub user_name: String,
}

/// Implementation of the builder pattern for the [Author] `struct`.
///
/// # Description
///
/// Use this object to partially build an [Author] object. For example:
///
/// ```rust
/// use lacoctelera::domain::{Author, AuthorBuilder};
///
/// let author = AuthorBuilder::default()
///     .set_name("Jane")
///     .set_surname("Doe")
///     .set_shareable(true)
///     .build().unwrap();
/// ```
///
/// [AuthorBuilder::build] shall be called after all the member initializations. This method returns a [Result] if
/// some of the values given to the constructor don't comply with the restrictions defined to [Author]'s members.
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

impl std::default::Default for Author {
    /// Default constructor for [Author].
    ///
    /// # Description
    ///
    /// This constructor builds a new [Author] whose name is randomly generated using funny names and adjectives.
    fn default() -> Self {
        Author {
            id: Some(Uuid::now_v7()),
            name: Some(Generator::default().next().unwrap()),
            surname: None,
            email: None,
            shareable: Some(false),
            description: None,
            website: None,
            social_profiles: None,
        }
    }
}

impl Author {
    /// Constructor of the [Author] struct.
    ///
    /// # Descriptor
    ///
    /// All fields are optional
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Option<String>,
        name: Option<String>,
        surname: Option<String>,
        email: Option<String>,
        shareable: Option<bool>,
        description: Option<String>,
        website: Option<String>,
        social_profiles: Option<&[SocialProfile]>,
    ) -> Result<Self, DataDomainError> {
        let id = if id.is_some() {
            match Uuid::parse_str(&id.unwrap()) {
                Ok(id) => Some(id),
                Err(_) => return Err(DataDomainError::InvalidId),
            }
        } else {
            None
        };

        let author = Author {
            id,
            name,
            surname,
            email,
            shareable,
            description,
            website,
            social_profiles: social_profiles.map(Vec::from),
        };

        match author.validate() {
            Ok(_) => std::result::Result::Ok(author),
            Err(e) => Err(DataDomainError::InvalidParams { source: e }),
        }
    }

    pub fn id(&self) -> Option<String> {
        self.id.map(|id| id.to_string())
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn surname(&self) -> Option<&str> {
        self.surname.as_deref()
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn shareable(&self) -> bool {
        self.shareable.unwrap_or_default()
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

    pub fn build(self) -> Result<Author, DataDomainError> {
        Author::new(
            self.id,
            self.name,
            self.surname,
            self.email,
            Some(self.shareable),
            self.description,
            self.website,
            self.social_profiles.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn build_author_using_builder() {
        let author = AuthorBuilder::default().build().unwrap();
        assert_eq!(author.id, None);
        assert_eq!(author.name, None);
        assert_eq!(author.surname, None);
        assert_eq!(author.email, None);
        assert_eq!(author.shareable, Some(false));
        assert_eq!(author.description, None);
        assert_eq!(author.website, None);
        assert!(author.social_profiles.is_none());

        let id = Uuid::now_v7().to_string();

        let social_profiles = [
            SocialProfile {
                provider_name: "Facebook".into(),
                user_name: "janedoe".into(),
            },
            SocialProfile {
                provider_name: "Instragram".into(),
                user_name: "janedoe".into(),
            },
        ];

        let author = AuthorBuilder::default()
            .set_id(&id)
            .set_name("Jane")
            .set_surname("Doe")
            .set_email("jane_doe@mail.com")
            .set_description("An unknown person.")
            .set_website("http://janedoe.com")
            .set_shareable(true)
            .set_social_profiles(&social_profiles)
            .build()
            .expect("Failed to build author");

        assert_eq!(author.id().unwrap(), id);
        assert_eq!(author.name().unwrap(), "Jane");
        assert_eq!(author.surname().unwrap(), "Doe");
        assert_eq!(author.email().unwrap(), "jane_doe@mail.com");
        assert_eq!(author.shareable(), true);
        assert_eq!(author.description().unwrap(), "An unknown person.");
        assert_eq!(author.website().unwrap(), "http://janedoe.com");
        assert_eq!(author.social_profiles().unwrap(), social_profiles);
    }

    #[test]
    fn build_author_using_new() {
        let id = Uuid::now_v7().to_string();
        let social_profiles = [
            SocialProfile {
                provider_name: "Facebook".into(),
                user_name: "janedoe".into(),
            },
            SocialProfile {
                provider_name: "Instragram".into(),
                user_name: "janedoe".into(),
            },
        ];

        let author = Author::new(
            Some(id.clone()),
            Some("Jane".to_string()),
            Some("Doe".to_string()),
            Some("jane_doe@mail.com".to_string()),
            Some(true),
            Some("An unknown person.".to_string()),
            Some("http://janedoe.com".to_string()),
            Some(&social_profiles),
        )
        .expect("Failed to create new instance of Author using new.");

        assert_eq!(author.id().unwrap(), id);
        assert_eq!(author.name().unwrap(), "Jane");
        assert_eq!(author.surname().unwrap(), "Doe");
        assert_eq!(author.email().unwrap(), "jane_doe@mail.com");
        assert_eq!(author.shareable(), true);
        assert_eq!(author.description().unwrap(), "An unknown person.");
        assert_eq!(author.website().unwrap(), "http://janedoe.com");
        assert_eq!(author.social_profiles().unwrap(), social_profiles);
    }

    #[test]
    fn build_author_using_wrong_id() {
        let author = AuthorBuilder::default().set_id("Wrong_ID").build();
        assert!(author.is_err());
        let author = AuthorBuilder::default().set_id("191919-010010-022").build();
        assert!(author.is_err());
    }

    #[test]
    fn build_author_using_wrong_text_length() {
        let author = AuthorBuilder::default().set_name("J").build();
        assert!(author.is_err());

        let author = AuthorBuilder::default().set_surname("D").build();
        assert!(author.is_err());

        let author = AuthorBuilder::default().set_website("janedoe.com").build();
        assert!(author.is_err());

        let author = AuthorBuilder::default()
            .set_email("janedoe<at>mail.com")
            .build();
        assert!(author.is_err());

        let author = AuthorBuilder::default()
            .set_description(&"dummy string".repeat(300))
            .build();
        assert!(author.is_err());
    }
}
