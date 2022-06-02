use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::lazy_static::lazy_static;
use ::regex::Regex;
use ::sha2::Digest;
use ::sha2::Sha256;

lazy_static! {
    static ref INVALID_CHARS: Regex = Regex::new("[^a-zA-Z0-9 _-]").unwrap();
    static ref SQUASH_CHARS: Regex = Regex::new("[ _-]+").unwrap();
    static ref INVALID_EDGES: Regex = Regex::new("(^[0-9_]*|[0-9_]+$)").unwrap();
}

/// Turn a text into a safe filename, trying to keep it unique.
pub fn unique_filename(text: &str) -> String {
    let clean = INVALID_CHARS.replace_all(text, "_");
    let squash = SQUASH_CHARS.replace_all(clean.as_ref(), "_");
    let trim = INVALID_EDGES.replace_all(squash.as_ref(), "");
    let short: String = trim.as_ref().chars().take(32).collect();
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    let hash = encode_config(hash_out, URL_SAFE_NO_PAD)[..20].to_ascii_lowercase();
    format!("{short}_{hash}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_legal_filename() {
        let res = unique_filename("Hello world");
        assert_eq!(res, "Hello_world_zoyiygcyaow6gjvnihtt");
    }

    #[test]
    fn long_illegal_filename() {
        let res = unique_filename(
            " _ hello WORLD hello world 你好 你好 你好 hello world- !!! !@#$%^& bye 123",
        );
        assert_eq!(res, "hello_WORLD_hello_world_hello_wo_zc4zyofxrnr1onvipg5w");
    }
}
