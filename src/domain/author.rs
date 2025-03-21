// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Data objects related to Authors.

use crate::{domain::DataDomainError, validate_id};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
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
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, IntoParams, Validate)]
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
    /// URL of the social network. 80 chars max.
    #[validate(length(max = 80))]
    pub website: String,
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
            name: None,
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

    pub fn mute_private_data(&mut self) {
        if !self.shareable() {
            self.email = None;
            self.description = None;
        }
    }

    pub fn enable_sharing(&mut self) {
        self.shareable = Some(true);
    }

    pub fn disable_sharing(&mut self) {
        self.shareable = Some(false);
    }

    /// Update the internal attributes using another [Author] object.
    ///
    /// # Description
    ///
    /// This method takes as reference another [Author] object, and replaces the internal values, which are also
    /// present in the given reference, using the values from the reference. This method is meant to implement a
    /// PATCH logic.
    pub fn update_from(&mut self, update: &Author) {
        if update.id().is_some() {
            self.id = Some(Uuid::parse_str(&update.id().unwrap()).unwrap());
        }
        if update.name().is_some() {
            self.name = Some(update.name().unwrap().into());
        }
        if update.surname().is_some() {
            self.surname = Some(update.surname().unwrap().into());
        }
        if update.email().is_some() {
            self.email = Some(update.email().unwrap().into());
        }
        if update.description().is_some() {
            self.description = Some(update.description().unwrap().into());
        }
        if update.website().is_some() {
            self.website = Some(update.website().unwrap().into());
        }
        if update.social_profiles().is_some() {
            self.social_profiles = Some(Vec::from(update.social_profiles().unwrap()));
        }
    }
}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.surname == other.surname
            && self.email == other.email
            && self.shareable == other.shareable
            && self.description == other.description
            && self.social_profiles == other.social_profiles
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
                website: "a web site".into(),
            },
            SocialProfile {
                provider_name: "Instragram".into(),
                website: "a web site".into(),
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
    fn build_author() {
        let mut author = Author::default();
        assert!(Uuid::parse_str(&author.id().unwrap()).is_ok());
        assert_eq!(author.name(), None);

        assert!(!author.shareable());
        author.enable_sharing();
        assert!(author.shareable());
        author.disable_sharing();
        assert!(!author.shareable());

        let id = Uuid::now_v7().to_string();
        let social_profiles = [
            SocialProfile {
                provider_name: "Facebook".into(),
                website: "a web site".into(),
            },
            SocialProfile {
                provider_name: "Instragram".into(),
                website: "a web site".into(),
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

    #[test]
    fn mute_fields() {
        let id = Uuid::now_v7().to_string();
        let social_profiles = [
            SocialProfile {
                provider_name: "Facebook".into(),
                website: "a web site".into(),
            },
            SocialProfile {
                provider_name: "Instragram".into(),
                website: "a web site".into(),
            },
        ];
        let mut author = Author::new(
            Some(id.clone()),
            Some("Jane".to_string()),
            Some("Doe".to_string()),
            Some("jane_doe@mail.com".to_string()),
            Some(false),
            Some("An unknown person.".to_string()),
            Some("http://janedoe.com".to_string()),
            Some(&social_profiles),
        )
        .expect("Failed to create new instance of Author using new.");

        author.mute_private_data();

        assert_eq!(author.id().unwrap(), id);
        assert_eq!(author.name().unwrap(), "Jane");
        assert_eq!(author.surname().unwrap(), "Doe");
        assert_eq!(author.email(), None);
        assert_eq!(author.shareable(), false);
        assert_eq!(author.description(), None);
        assert_eq!(author.website().unwrap(), "http://janedoe.com");
        assert_eq!(author.social_profiles().unwrap(), social_profiles);
    }

    #[test]
    fn modify_from() {
        // Let's build the author under test
        let id = Uuid::now_v7().to_string();
        let name = "Jane";
        let surname = "Doe";
        let email = "jane@mail.com";
        let website = "https://jane.com";
        let description = "A dummy description";
        let profiles = &[SocialProfile {
            provider_name: "None".into(),
            website: "https://none.com/jane".into(),
        }];

        let mut author = AuthorBuilder::default()
            .set_id(&id)
            .set_name(name)
            .set_surname(surname)
            .set_email(email)
            .set_description(description)
            .set_shareable(true)
            .set_website(website)
            .set_social_profiles(profiles)
            .build()
            .expect("Failed to build an author");

        // First test case: modify only some of the attributes.
        let author_dummy = AuthorBuilder::default()
            .set_name("Stripped")
            .set_surname("Zebra")
            .build()
            .expect("Failed to build an author");
        author.update_from(&author_dummy);

        assert_eq!(author.name, author_dummy.name);
        assert_eq!(author.surname, author_dummy.surname);
        assert_ne!(author.id, author_dummy.id);
        assert_ne!(author.email, author_dummy.email);
        assert_ne!(author.website, author_dummy.website);
        assert_ne!(author.description, author_dummy.description);
        assert_ne!(author.social_profiles, author_dummy.social_profiles);

        // Second test case: modify all the attributes but the ID.
        let name = "Juana";
        let surname = "Cierva";
        let email = "juana@mail.com";
        let website = "https://juana.com";
        let description = "Una descripción vana";
        let profiles = &[SocialProfile {
            provider_name: "None".into(),
            website: "https://none.com/juana".into(),
        }];

        let author_spa = AuthorBuilder::default()
            .set_id(&id)
            .set_name(name)
            .set_surname(surname)
            .set_email(email)
            .set_description(description)
            .set_shareable(true)
            .set_website(website)
            .set_social_profiles(profiles)
            .build()
            .expect("Failed to build an author");

        author.update_from(&author_spa);

        assert_eq!(author, author_spa);
    }
}
