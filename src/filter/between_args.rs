use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use ::clap::Parser;

use ::regex::Regex;

#[derive(Parser, Debug)]
#[command(
    name = "between",
    about = "Select all lines between two matches"
)]
pub struct BetweenArgs {
    #[arg(short = 'f', long, default_value = ".")]
    /// Start collecting lines when this expression matches
    pub from: Regex,
    #[arg(short = 't', long)]
    /// Stop collecting lines when this expression matches
    pub to: Option<Regex>,
    #[arg(short = 'F', long, default_value = "include")]
    /// How to handle the matched --from line, include [i] of exclude [e]
    pub from_handling: MatchHandling,
    #[arg(short = 'T', long, default_value = "exclude")]
    /// How to handle the matched --to line, include [i] of exclude [e]
    pub to_handling: MatchHandling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchHandling {
    Include,
    Exclude,
}

impl FromStr for MatchHandling {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text.to_lowercase().as_str() {
            "include" | "i" => MatchHandling::Include,
            "skip" | "s" | "exclude" | "e" => MatchHandling::Exclude,
            other => return Err(format!("unknown match handling mode: {}", other)),
        })
    }
}

impl fmt::Display for MatchHandling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            MatchHandling::Include => "include",
            MatchHandling::Exclude => "exclude",
        })
    }
}

#[test]
fn test_cli_args() {
    BetweenArgs::try_parse_from(&["cmd", "--from", ".*"]).unwrap();
    BetweenArgs::try_parse_from(&["cmd", "--to", "^END$", "-T", "i"]).unwrap();
    BetweenArgs::try_parse_from(&["cmd", "-f", ".*", "-F", "i", "-t", "^END$", "-T", "s"]).unwrap();
}
