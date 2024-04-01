use ::std::str::FromStr;

use ::lazy_static::lazy_static;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;

lazy_static! {
    static ref PROFILE_RE: Regex = Regex::new(r"^!?\w[\w/_\-]*\w?$").unwrap();
}

#[derive(
    Debug, derive_more::Display, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct Profile {
    value: String,
}

impl Profile {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if !PROFILE_RE.is_match(&value) {
            return Err(
                "profile name must be alphanumeric and may also contain: / - _".to_string(),
            );
        }
        Ok(Profile { value })
    }
}

impl From<Profile> for String {
    fn from(profile: Profile) -> String {
        format!("{}", profile)
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Profile::new(value)
    }
}

impl TryFrom<String> for Profile {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Profile::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_valid() {
        assert!(Profile::new("!my/profile").is_ok());
    }

    #[test]
    fn profile_invalid() {
        assert!(Profile::new("my.profile").is_err());
    }
}
