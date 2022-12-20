use ::clap::Parser;
use ::std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "namesafe",
    about = "Convert each line to a string that is safe for names (no whitespace or special characters, not too long)."
)]
pub struct NamesafeArgs {
    /// Allow non-ascii letters (but no non-letter symbols).
    #[arg(
        parse(from_flag = Charset::from_allow),
        short = 'u',
        long = "allow-unicode",
    )]
    pub charset: Charset,
    /// In which cases to include a hash in the name ([a]lways, [c]hanged, too-[l]ong, [n]ever).
    #[arg(
        short = 'x',
        long = "hash",
        default_value = "changed",  //TODO @mverleg: not sure why Default impl doesn't work
    )]
    pub hash_policy: HashPolicy,
    /// Maximum number of characters in the cleaned line (min 8).
    #[arg(short = 'l', long = "max-length", default_value = "32")]
    pub max_length: u32,
    /// If the line appears to contain an filename extension (max 4 chars), preserve it.
    #[arg(short = 'e', long = "extension")]
    pub keep_extension: bool,
    /// If the command has to be shortened, keep the end part instead of the start.
    #[arg(short = 'E', long = "keep-tail")]
    pub keep_tail: bool,
    /// Do not fail if there are no input lines.
    #[arg(short = '0', long = "allow-empty")]
    pub allow_empty: bool,
    /// Expect exactly one input line. Fail if more. Fail if fewer unless --allow_empty.
    #[arg(short = '1', long = "single")]
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
            "never" | "n" => HashPolicy::Never,
            other => return Err(format!("unknown hash policy: {}", other)),
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
            Charset::AllowUnicode => symbol.is_alphanumeric() || symbol == '-' || symbol == '_',
            Charset::AsciiOnly => {
                ('a'..='z').contains(&symbol)
                    || ('A'..='Z').contains(&symbol)
                    || ('0'..='9').contains(&symbol)
                    || symbol == '-'
                    || symbol == '_'
            }
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
            keep_tail: false,
            allow_empty: false,
            single_line: false,
        }
    }
}

#[test]
fn test_cli_args() {
    NamesafeArgs::try_parse_from(&["cmd", "--help"]).unwrap();
}
