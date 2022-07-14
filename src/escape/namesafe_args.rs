use std::str::FromStr;
use ::clap::StructOpt;

#[derive(StructOpt, Debug, Default)]
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
        help = "In which cases to include a hash in the name."
    )]
    pub hash_policy: HashPolicy,
    #[structopt(
        short = 'l',
        long = "max-length",
        help = "Maximum number of characters in the cleaned line."
    )]
    pub max_length: u32,
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
    #[default]
    AllowUnicode,
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
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    NamesafeArgs::into_app().debug_assert()
}
