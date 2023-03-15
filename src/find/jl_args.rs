use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::Parser;
use ::regex::Regex;

#[derive(Parser, Debug)]
#[command(name = "jl", about = "A mix of ls and find that outputs json (use jq)")]
pub struct JlArgs {
    #[arg(short = 'n', long, default_value = "1")]
    /// Maximum directory depth to recurse into
    pub max_depth: u32,
    #[arg(short = 'P', long)]
    /// Do not recurse into symlinked directories
    pub no_recurse_symlinks: bool,
    #[arg(short = 'L', long)]
    /// Return one entry per line, not wrapping into a list
    pub entry_per_lines: bool,
    #[arg(short = 'f', long)]
    /// Regular expression to filter filenames by (default: return everything) (only names)
    pub filter: Option<Regex>,
    #[arg(short = 'e', long = "on-error", default_value = "fail-at-end")]
    /// What to do when failing to read a file
    pub on_error: ErrorHandling,
    #[arg(short = 'h', long = "hash")]
    /// Add a hash of file content to the result
    pub do_hash: bool,
    #[arg(short = 'd', long)]
    /// Only return directories, not files
    pub only_dirs: bool,
    #[arg(short = 'D', long, conflicts_with = "only_dirs")]
    /// Do not return directories, only files
    pub no_dirs: bool,
    #[arg(default_value = "./")]
    /// Directory to search in
    pub root: PathBuf,
}

#[test]
fn test_cli_args() {
    JlArgs::try_parse_from(&["jl"]).unwrap();
    JlArgs::try_parse_from(&["jl", "-d", "2", "-f", "^.*\\.java$", "-P", "-L", "/tmp", "-D"]).unwrap();
}

impl Default for JlArgs {
    fn default() -> Self {
        JlArgs {
            max_depth: 1,
            no_recurse_symlinks: false,
            entry_per_lines: false,
            filter: None,
            on_error: Default::default(),
            do_hash: false,
            only_dirs: false,
            no_dirs: false,
            root: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ErrorHandling {
    Abort,
    #[default]
    FailAtEnd,
    Warn,
    Ignore,
}

impl FromStr for ErrorHandling {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text.to_lowercase().as_str() {
            "abort" | "a" => ErrorHandling::Abort,
            "fail-at-end" | "fail" | "f" => ErrorHandling::FailAtEnd,
            "warn" | "w" => ErrorHandling::Warn,
            "ignore" | "silence" | "i" => ErrorHandling::Ignore,
            other => return Err(format!("unknown error handling mode: {}", other)),
        })
    }
}