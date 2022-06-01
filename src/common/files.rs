use ::lazy_static::lazy_static;
use ::regex::Regex;
use ::sha2::Digest;
use ::sha2::Sha256;
use base64::{encode_config, URL_SAFE_NO_PAD};

lazy_static! {
    static ref INVALID_CHARS: Regex = Regex::new("[^ a-zA-Z0-9_-]").unwrap();
    static ref SQUASH_CHARS: Regex = Regex::new("[\\s-_]+").unwrap();
}

/// Turn a text into a safe filename, trying to keep it unique.
pub fn to_filename(text: &str) -> String {
    let clean = INVALID_CHARS.replace_all(text, "_");
    let squash = SQUASH_CHARS.replace_all(clean.as_ref(), "_");
    let short = squash.as_ref()[..32].to_owned();
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    let hash = &encode_config(hash_out, URL_SAFE_NO_PAD)[..20];
    format!("{short}_{hash}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_filename() {
        let res = to_filename("hello world");
        assert_eq!(res, "");
    }
}
