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
pub struct JlArgs {
    #[arg(short = 'd', long, default_value = "1")]
    /// Maximum directory depth to recurse into
    pub max_depth: u32,
    #[arg(short = 'P', long)]
    /// Do not recurse into symlinked directories
    pub no_recurse_symlinks: bool,
    #[arg(short = 'L', long)]
    /// Return one entry per line, not wrapping into a list
    pub entry_per_lines: bool,
    #[arg()]
    /// Regular expression to filter by (default: return everything)
    pub pattern: Option<Regex>,
}

#[test]
fn test_cli_args() {
    JlArgs::try_parse_from(&["cmd", "-d", "2", "^.*\\.java$"]).unwrap();
}
