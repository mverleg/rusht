use ::std::path::PathBuf;

use ::clap::Parser;
use ::regex::Regex;

use crate::find::dir_with_args::OnErr;

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
    #[arg(short = 'c', long = "content")]
    /// Content pattern that must match inside the file
    pub contents: Vec<Regex>,
    #[arg(short = 'P', long = "not-path")]
    /// Opposite of -p; file only matches if full path does NOT match this pattern
    pub not_paths: Vec<Regex>,
    #[arg(short = 'C', long = "not-content")]
    /// Opposite of -t; file only matches if content does NOT match this pattern
    pub not_contents: Vec<Regex>,
}

#[test]
fn test_cli_args() {
    // Test basic path pattern matching
    FilesWithArgs::try_parse_from(&["cmd", "-r", ".", "-l", "6", "-p", ".*\\.txt", "-x=silent"]).unwrap();
    
    // Test content pattern matching with negative path pattern
    FilesWithArgs::try_parse_from(&["cmd", "-r", ".", "-c", "TODO", "-P", "test.*"]).unwrap();
    
    // Test multiple patterns
    FilesWithArgs::try_parse_from(&["cmd", "-p", ".*\\.rs", "-p", ".*\\.toml", "-c", "async", "-C", "deprecated"]).unwrap();
}

