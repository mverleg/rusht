use ::clap::Parser;
use ::regex::Regex;

#[derive(Parser, Debug)]
#[command(
    name = "grab",
    about = "Filter lines by regular expression, keeping only the matching capture group."
)]
pub struct GrabArgs {
    /// Regular expression to match. Returns the capture group if any, or the whole match otherwise.
    #[arg()]
    pub pattern: Regex,
    #[arg(short = 'f', long = "first-match-only")]
    /// Only print the first match of the pattern per line, even if it matches multiple times.
    ///
    /// Note the difference with --first-capture-only, see help note there.
    pub first_match_only: bool,
    /// Only print the first capture group per pattern match, even if there are multiple groups in the pattern.
    /// {n}* '(a+)(b+)?' matches twice in 'aaba' with one capture each.
    /// {n}* '(a+)(b+)?' matches once in 'aabcdef' but has two captures.
    #[arg(short = '1', long = "first_capture_only")]
    pub first_capture_only: bool,
    /// Keep the full line if it does not match the pattern
    #[arg(short = 'k', long)]
    pub keep_unmatched: bool,
    /// Maximum number of matching lines
    #[arg(short = 'n', long)]
    pub max_lines: Option<u32>,
}

impl Default for GrabArgs {
    fn default() -> Self {
        GrabArgs {
            pattern: Regex::new(".*").unwrap(),
            first_match_only: false,
            first_capture_only: false,
            keep_unmatched: false,
            max_lines: None,
        }
    }
}

#[async_std::test]
async fn test_cli_args() {
    GrabArgs::try_parse_from(&["cmd", "-f1kn", "5", "^.{5}$"]).unwrap();
}
