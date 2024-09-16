//! Data objects related to tags.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

// Regex to validate an Uuid.
static RE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z_]{2,}$").unwrap());

/// Tag data object.
///
/// # Description
///
/// Tags allow a fine-grain organisation of the recipes in the DB. They allow to stablish relationships between
/// different recipes, and they can be used to offer recipe suggestions to clients.
///
/// Tags are identified by a single word identifier with a minimum length of 2 and maximum of 20 characters.
/// Tags are not case sensitive, and capital letters will be converted to lower case when using any of the provided
/// methods.
///
/// The only special character that is allowed to identify a tag is: `_`.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, Validate, PartialEq)]
pub struct Tag {
    #[validate(custom(function = "validate_identifier"), length(min = 2, max = 20))]
    pub identifier: String,
}

impl Tag {
    pub fn new(tagname: &str) -> Result<Self, ValidationError> {
        let tag = Tag {
            identifier: tagname.to_ascii_lowercase(),
        };

        match tag.validate() {
            Ok(_) => Ok(tag),
            Err(_) => Err(ValidationError::new("2")),
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

/// Custom function to validate a string used to build a [Tag].
fn validate_identifier(value: &str) -> Result<(), ValidationError> {
    let haystack = value.to_lowercase();
    if RE_TAG.is_match(&haystack) {
        Result::Ok(())
    } else {
        Err(ValidationError::new("2"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case("A(tag)")]
    #[case("")]
    #[case("Averylongtagnamethatshallprovokeanerror")]
    #[case("anemoji❌")]
    fn wrong_string_fail_to_build_a_tag(#[case] input: &str) {
        assert!(Tag::new(input).is_err())
    }

    #[rstest]
    #[case("A_tag")]
    #[case("Ta")]
    #[case("customTag")]
    fn valid_string_succeed_to_build_a_tag(#[case] input: &str) {
        assert!(Tag::new(input).is_ok())
    }

    #[rstest]
    #[case("customTag")]
    #[case("ALLCAPITALLETTERSTAG")]
    fn tags_get_converted_to_lowercase(#[case] input: &str) {
        assert_eq!(
            Tag::new(input).expect("Failed to build a tag").identifier,
            input.to_lowercase()
        )
    }
}
