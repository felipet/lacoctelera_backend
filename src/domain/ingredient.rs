// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::bail;
use core::fmt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::convert::{Into, TryFrom};
use std::error::Error;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use super::DataDomainError;

/// This value is set in the DB's schema definition (VARCHAR(40)).
const MAX_NAME_LENGTH: usize = 40;
/// This value is set in the DB's schema definition (VARCHAR(255)).
const MAX_DESC_LENGTH: usize = 255;

/// Types of ingredients of teh `Cocktail` data base.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, ToSchema)]
pub enum IngCategory {
    /// Spirit ingredients, such as rum, liquors and so.
    Spirit,
    /// Bitter ingredients, such as Angostura.
    Bitter,
    /// Soft-drink ingredients, such as soda water, Fanta, Coke, etc.
    #[serde(alias = "soft_drink")]
    SoftDrink,
    /// Garnish ingredients, such a lemon's peel.
    Garnish,
    /// Category for ingredients whose type does not match any of the other types.
    Other,
}

/// Object that represents an Ingredient of the `Cocktail` data base.
///
/// # Description
///
/// An ingredient represents the elements that are contained by a Cocktail's recipe.
/// This object only represents the ingredients as a model for being included in
/// individual recipes. Hence no information related to quantities or any other information
/// that joins an ingredient with a recipe  is included as an attribute of this object.
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Ingredient {
    id: Option<Uuid>,
    name: String,
    category: IngCategory,
    description: Option<String>,
}

impl Ingredient {
    /// Builds a new [Ingredient] performing checks over the input parameters.
    ///
    /// # Description
    ///
    /// The implementation checks that the given _name_ value meets the following requirements:
    /// - The length of the name doesn't exceeds 40 characters.
    /// - The name is composed of alphanumeric characters plus the special character `%`.
    /// - The name does not contain the following forbidden characters: `[`, `<`, `>`, `;`,
    ///   `{`, `}`, `]`.
    ///
    /// # Arguments
    ///
    /// - _name_ will be used as [Ingredient::name].
    /// - _category_ will be used as [Ingredient::category]. Use [IngCategory::Other] when no
    ///   needed.
    /// - _desc_ will be used as [Ingredient::desc]. Pass `None` when no description was provided
    ///   along the Ingredient's name.
    ///
    /// # Return
    ///
    /// A new [Ingredient] when the input parameters comply the format rules, an error otherwise
    /// that contains a message with information about the broken format rule.
    pub fn parse(
        id: Option<&str>,
        name: &str,
        category: &str,
        description: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let id = if let Some(id) = id {
            Some(Uuid::parse_str(id).map_err(|e| {
                error!("Failed to parse an UUID from {e}");
                Box::new(DataDomainError::InvalidId)
            })?)
        } else {
            None
        };

        let name = match Ingredient::check_name(name) {
            Ok(name) => name,
            Err(e) => {
                error!("Invalid name ({e}) given to parse an Ingredient");
                return Err(Box::new(DataDomainError::InvalidFormData));
            }
        };

        let category = category.try_into()?;

        let description = match description {
            Some(desc) => match Ingredient::check_desc(desc) {
                Ok(desc) => Some(desc),
                Err(e) => {
                    error!("Invalid description ({e}) given to parse an Ingredient");
                    return Err(Box::new(DataDomainError::InvalidFormData));
                }
            },
            None => None,
        };

        Ok(Self {
            name,
            category,
            description,
            id,
        })
    }

    /// Get the Ingredient's  name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the Ingredient's category as a value of the `Enum` [IngCategory].
    pub fn category(&self) -> IngCategory {
        self.category
    }

    /// Get the description of the Ingredient. Wrapped to allow empty descriptions.
    pub fn desc(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the ingredient's ID in the `Cocktail` data base.
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    /// Set the ID of the ingredient in the `Cocktail` data base.
    pub fn set_id(&mut self, id: Uuid) {
        self.id = Some(id);
    }

    /// Check that a string is valid as [Ingredient::name].
    ///
    /// # Description
    ///
    /// This internal method performs a series of checks against a given string in order
    /// to detect the violation of some design rule for [Ingredient::name]. The following
    /// is checked:
    /// - The string does not exceed 40 characters.
    /// - The string's format is a name that might includes numbers and/or the symbol `%`.
    /// - The string does not contain the following forbidden characters: `[`, `<`, `>`, `;`,
    ///   `{`, `}`, `]`.
    ///
    /// # Arguments
    ///
    /// A string that contains the name to be checked.
    ///
    /// # Return
    ///
    /// A  `Result` enum with:
    /// - A `String` on success that is an owned copy of the string given as argument.
    /// - Otherwise, an error that contains a message that informs what rule was violated.
    fn check_name(name: &str) -> Result<String, anyhow::Error> {
        // Avoid processing long strings that exceed the maximum allowed.
        if name.len() > MAX_NAME_LENGTH {
            bail!("The length of the given Ingredient's name exceeds {MAX_NAME_LENGTH} characters.")
        }

        // Regex for the validation of usual strings composed by words, numbers
        // and the symbol %.
        let validation_regex = [
            Regex::new(r"^[[:alpha:]]{1,}? +[[:alpha:]]{0,}?").unwrap(),
            Regex::new(r"^[[:alpha:]]{1,}?[[ :alpha:\d]%]{0,}").unwrap(),
        ];

        // Apply the previous regex to the input value.
        let validated = validation_regex
            .iter()
            .fold(false, |acc, r| acc | r.is_match(name));

        if !validated {
            bail!("The given Ingredient's name ({name}) has an invalid format.")
        }

        // Finally, look for forbidden characters in the input string.
        let forbidden_chars = Regex::new(r"[;<>`\{\}]").unwrap();

        if forbidden_chars.is_match(name) {
            bail!("The given Ingredient's name ({name}) contains invalid characters.")
        } else {
            Ok(String::from(name))
        }
    }

    /// Check that a string is valid as [Ingredient::desc].
    ///
    /// # Description
    ///
    /// A very basic check is performed: ensure that the length doesn't exceeds the
    /// maximum allowed (255 characters).
    ///
    /// # Arguments
    ///
    /// A string that contains the description of an `Ingredient`.
    ///
    /// # Return
    ///
    /// A `Result` enum with:
    /// - A `String` on success that contains an owned version of the string given as
    ///   argument.
    /// - Otherwise, an error that contains a message that informs about the violated rule.
    fn check_desc(desc: &str) -> Result<String, anyhow::Error> {
        // Avoid processing long strings that exceed the maximum allowed.
        if desc.len() > MAX_DESC_LENGTH {
            bail!("The length of the given string exceeds {MAX_DESC_LENGTH} characters.")
        }

        Ok(String::from(desc))
    }
}

impl PartialEq for Ingredient {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.category == other.category
            && self.description == other.description
    }
}

impl TryFrom<String> for IngCategory {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&str> for IngCategory {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Spirit" | "spirit" => Ok(IngCategory::Spirit),
            "Bitter" | "bitter" => Ok(IngCategory::Bitter),
            "SofDrink" | "softdrink" | "soft_drink" => Ok(IngCategory::SoftDrink),
            "Garnish" | "garnish" => Ok(IngCategory::Garnish),
            "Other" | "other" => Ok(IngCategory::Other),
            _ => bail!("Invalid ingredient category."),
        }
    }
}

impl fmt::Display for IngCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl IngCategory {
    pub fn to_str(&self) -> &str {
        match self {
            IngCategory::Bitter => "bitter",
            IngCategory::Garnish => "garnish",
            IngCategory::SoftDrink => "soft_drink",
            IngCategory::Spirit => "spirit",
            IngCategory::Other => "other",
        }
    }
}

impl fmt::Debug for Ingredient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'name': '{}', 'category': '{}', 'desc': '{}'",
            self.name,
            self.category,
            self.description.as_deref().unwrap_or("")
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("vodka", true)]
    #[case("white rum", true)]
    #[case("vodka 80", true)]
    #[case("liquor 20%", true)]
    #[case("liquor 40% coconut", true)]
    #[case("my new fancy rum 100", true)]
    #[case("<insert>", false)]
    #[case("name;`another string`", false)]
    #[case("very long text string should not be accepted", false)]
    fn convert_names_to_ingredients(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(Ingredient::check_name(input).is_ok(), expected);
    }
}
