use anyhow::bail;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
pub enum IngCategory {
    Spirit,
    Bitter,
    SoftDrink,
    Garnish,
    Other,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Ingredient {
    pub name: String,
    pub category: Option<IngCategory>,
    pub desc: Option<String>,
}

/// This value is set in the DB's schema definition (VARCHAR(40)).
const MAX_NAME_LENGTH: usize = 40;

impl From<String> for IngCategory {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Spirit" | "spirit" => IngCategory::Spirit,
            "Bitter" | "bitter" => IngCategory::Bitter,
            "SofDrink" | "softdrink" => IngCategory::SoftDrink,
            "Garnish" | "garnish" => IngCategory::Garnish,
            _ => IngCategory::Other,
        }
    }
}

/// Instantiate a new [Ingredient] using a `String` that will be used as [Ingredient::name].
///
/// # Description
///
/// The implementation checks that the given value meets the following requirements:
/// - The length of the name doesn't exceeds 40 characters.
/// - The name is composed of alphanumeric characters plus the special character `%`.
///
/// The following characters are forbidden: _;_ , _<_ , _>_ , _{_ , _}_ , _`_ .
impl TryFrom<String> for Ingredient {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Avoid processing long strings that exceed the maximum allowed.
        if value.len() > MAX_NAME_LENGTH {
            bail!("The length of the given string exceeds {MAX_NAME_LENGTH} characters.")
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
            .fold(false, |acc, r| acc | r.is_match(&value));

        if !validated {
            bail!("Wrong format of the value ({value}) given as the Ingredient name")
        }

        // Finally, look for forbidden characters in the input string.
        let forbidden_chars = Regex::new(r"[;<>`\{\}]").unwrap();

        if forbidden_chars.is_match(&value) {
            bail!("Invalid name: {value} to build an Ingredient")
        } else {
            Ok(Ingredient {
                name: value.to_owned(),
                category: None,
                desc: None,
            })
        }
    }
}

/// The same restrictions applied by `TryFrom<String>` are applied to this implementation.
impl TryFrom<&str> for Ingredient {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_owned().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::Ingredient;
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
        assert_eq!(Ingredient::try_from(input).is_ok(), expected);
    }
}
