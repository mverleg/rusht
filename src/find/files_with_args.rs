use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::Parser;
use ::regex::Regex;

#[derive(Parser, Debug, Default)]
#[command(
    name = "files_with",
    about = "Find files that match certain patterns.",
    long_about = "Find files that match certain patterns. Only supports utf8, sensible filenames."
)]
pub struct FilesWithArgs {
    #[arg(short = 'l', long, default_value = "10000")]
    /// Maximum directory depth to recurse into
    pub max_depth: u32,
    #[arg(short = 'x', long = "on-error", default_value = "warn")]
    /// What to do when an error occurs: [w]arn, [a]bort or [i]gnore
    pub on_err: OnErr,
    #[arg(short = 'r', long = "root", default_value = ".")]
    /// Root directories to start searching from (multiple allowed)
    pub roots: Vec<PathBuf>,
    #[arg(short = 'p', long = "path")]
    /// Full path pattern that must match
    pub paths: Vec<Regex>,
    #[arg(short = 't', long = "content")]
    /// Content pattern that must match inside the file
    pub contents: Vec<Regex>,
    #[arg(short = 'P', long = "not-path")]
    /// Opposite of -p; file only matches if full path does NOT match this pattern
    pub not_paths: Vec<Regex>,
    #[arg(short = 'T', long = "not-content")]
    /// Opposite of -t; file only matches if content does NOT match this pattern
    pub not_contents: Vec<Regex>,
}

#[test]
fn test_cli_args() {
    FilesWithArgs::try_parse_from(&["cmd", "-r", ".", "-l", "6", "-f", ".*\\.txt", "-x=silent", ]).unwrap();
    FilesWithArgs::try_parse_from(&["cmd", "-r", ".", "-t", "TODO", "-F", "test.*", ]).unwrap();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnErr {
    #[default]
    Warn,
    Abort,
    Ignore,
}

impl FromStr for OnErr {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "w" | "warn" => OnErr::Warn,
            "a" | "abort" | "exit" | "stop" => OnErr::Abort,
            "i" | "ignore" | "silent" | "skip" => OnErr::Ignore,
            _ => return Err(format!("did not understand error handling strategy '{}', try '[w]arn', '[a]bort' or '[i]gnore'", value)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_value() {
        let range = IntRange::from_str("1024").unwrap();
        assert!(range.is_provided());
        assert!(range.includes(1024));
        assert!(!range.includes(1023));
        assert!(!range.includes(1025));
    }

    #[test]
    fn parse_lower_bound() {
        let range = IntRange::from_str("1024,").unwrap();
        assert!(range.is_provided());
        assert!(!range.includes(1023));
        assert!(range.includes(1024));
        assert!(range.includes(1025));
    }

    #[test]
    fn parse_upper_bound() {
        let range = IntRange::from_str(",1024").unwrap();
        assert!(range.is_provided());
        assert!(range.includes(1023));
        assert!(range.includes(1024));
        assert!(!range.includes(1025));
    }

    #[test]
    fn parse_closed_range() {
        let range = IntRange::from_str("1024,2048").unwrap();
        assert!(range.is_provided());
        assert!(range.includes(1024));
        assert!(range.includes(1536));
        assert!(range.includes(2048));
        assert!(!range.includes(1023));
        assert!(!range.includes(2049));
    }
}