use ::std::fmt;
use ::std::fmt::Formatter;
use ::std::fs;
use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::builder::BoolishValueParser;
use ::clap::builder::TypedValueParser;
use ::clap::ArgAction;
use ::clap::Parser;
use ::regex::Regex;

#[derive(Parser, Debug, Default)]
#[command(name = "jl", about = "A mix of ls and find that outputs json")]
pub struct DirWithArgs {
    #[arg(short = 'd', long, default_value = "1")]
    /// Maximum directory depth to recurse into
    pub max_depth: u32,
    #[arg()]
    /// Regular expression to filter by (default: return everything)
    pub pattern: Option<Regex>,
}

#[test]
fn test_cli_args() {
    DirWithArgs::try_parse_from(&["cmd", "-d", "2", "^.*\\.java$"]).unwrap();
}
