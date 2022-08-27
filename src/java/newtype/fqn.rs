use ::std::str::FromStr;

use ::derive_more;
use ::lazy_static::lazy_static;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;

lazy_static! {
    static ref FQN_RE: Regex = Regex::new(r"^([a-zA-Z][a-zA-Z0-9_]*\.)*([a-zA-Z][a-zA-Z0-9_]*)(\.\$[a-zA-Z][a-zA-Z0-9_]*)*$").unwrap();
}

#[derive(Debug, derive_more::Display, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, )]
#[serde(try_from = "String", into = "String")]
pub struct FullyQualifiedName {
    value: String,
}

impl FullyQualifiedName {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if !FQN_RE.is_match(&value) {
            return Err("fuilly qualified class identifier should be e.g. 'com.company.path.ClassNaMe'".to_string());
        }
        Ok(FullyQualifiedName { value })
    }
}

impl From<FullyQualifiedName> for String {
    fn from(fqn: FullyQualifiedName) -> String {
        format!("{}", fqn)
    }
}

impl FromStr for FullyQualifiedName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        FullyQualifiedName::new(value)
    }
}

impl TryFrom<String> for FullyQualifiedName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        FullyQualifiedName::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fqn_top_cls_valid() {
        assert!(FullyQualifiedName::new("com.company.package.Class1Name").is_ok());
    }

    #[test]
    fn fqn_nested_cls_valid() {
        assert!(FullyQualifiedName::new("com.company.package.Top.$Nested1").is_ok());
        assert!(FullyQualifiedName::new("com.company.package.Top.$Nested1.$Nested2").is_ok());
    }

    #[test]
    fn fqn_no_package_valid() {
        assert!(FullyQualifiedName::new("MyClass0").is_ok());
    }

    #[test]
    fn invalid_name() {
        assert!(FullyQualifiedName::new("com.company.package.0Invalid").is_err());
    }

    #[test]
    fn invalid_path() {
        assert!(FullyQualifiedName::new(".com.company.package.Class").is_err());
        assert!(FullyQualifiedName::new("com..company.package.Class").is_err());
        assert!(FullyQualifiedName::new("com.company.package.Class.").is_err());
    }

    #[test]
    fn incorrect_nested() {
        assert!(FullyQualifiedName::new("$Nested").is_err());
        assert!(FullyQualifiedName::new("Class$$Nested").is_err());
    }
}
