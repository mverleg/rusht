use std::str::FromStr;
use ::clap::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "namesafe",
    about = "Convert each line to a string that is safe for names (no whitespace or special characters, not too long)."
)]
pub struct NamesafeArgs {
    #[structopt(
        parse(from_flag = Charset::from_allow),
        short = 'u',
        long = "allow-unicode",
        help = "Allow non-ascii letters (but no non-letter symbols)."
    )]
    pub charset: Charset,
    #[structopt(
        short = 'x',
        long = "hash",
        default_value = "changed",  //TODO @mverleg: not sure why Default impl doesn't work
        help = "In which cases to include a hash in the name ([a]lways, [c]hanged, too-[l]ong, [n]ever)."
    )]
    pub hash_policy: HashPolicy,
    #[structopt(
        short = 'l',
        long = "max-length",
        default_value = "32",
        help = "Maximum number of characters in the cleaned line (min 8)."
    )]
    pub max_length: u32,
    #[structopt(
        short = 'e',
        long = "extension",
        help = "If the line appears to contain an filename extension (max 4 chars), preserve it."
    )]
    pub keep_extension: bool,
    #[structopt(
        short = '0',
        long = "allow-empty",
        help = "Do not fail if there are no input lines."
    )]
    pub allow_empty: bool,
    #[structopt(
        short = '1',
        long = "single",
        help = "Expect exactly one input line. Fail if more. Fail if fewer unless --allow_empty."
    )]
    pub single_line: bool,
}
//TODO @mverleg: when to hash? (always, if changed, if too long, never)

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HashPolicy {
    Always,
    #[default]
    Changed,
    TooLong,
    Never,
}

impl HashPolicy {
    pub fn should_hash(&self, was_changed: bool, was_too_long: bool) -> bool {
        match self {
            HashPolicy::Always => true,
            HashPolicy::Changed => was_changed || was_too_long,
            HashPolicy::TooLong => was_too_long,
            HashPolicy::Never => false,
        }
    }
}

impl FromStr for HashPolicy {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text.to_lowercase().as_str() {
            "always" | "a" => HashPolicy::Always,
            "changed" | "c" => HashPolicy::Changed,
            "too-long" | "long" | "l" => HashPolicy::TooLong,
            "never" | "n" => HashPolicy::TooLong,
            other => return Err(format!("unknown hash policy: {}", other))
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    AllowUnicode,
    #[default]
    AsciiOnly,
}

impl Charset {
    fn from_allow(allow_unicode: bool) -> Self {
        if allow_unicode {
            Charset::AllowUnicode
        } else {
            Charset::AsciiOnly
        }
    }

    pub fn is_allowed(&self, symbol: char) -> bool {
        match self {
            Charset::AllowUnicode => symbol.is_alphanumeric() ||
                symbol == '-' || symbol == '_',
            Charset::AsciiOnly => ('a' <= symbol && symbol <= 'z') ||
                ('A' <= symbol && symbol <= 'Z') ||
                ('0' <= symbol && symbol <= '9') ||
                symbol == '-' || symbol == '_'
        }
    }
}

impl Default for NamesafeArgs {
    fn default() -> Self {
        NamesafeArgs {
            charset: Default::default(),
            hash_policy: Default::default(),
            max_length: 32,
            keep_extension: false,
            allow_empty: false,
            single_line: false
        }
    }
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    NamesafeArgs::into_app().debug_assert()
}
