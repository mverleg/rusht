use ::clap::StructOpt;
use ::regex::Regex;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grab",
    about = "Filter lines by regular expression, keeping only the matching capture group."
)]
pub struct GrabArgs {
    /// Regular expression to match. Returns the capture group if any, or the whole match otherwise.
    #[structopt()]
    pub pattern: Regex,
    #[structopt(short = 'f', long = "first-match-only")]
    /// Only print the first match of the pattern per line, even if it matches multiple times.
    ///
    /// Note the difference with --first-capture-only, see help note there.
    pub first_match_only: bool,
    /// Only print the first capture group per pattern match, even if there are multiple groups in the pattern.
    /// {n}* '(a+)(b+)?' matches twice in 'aaba' with one capture each.
    /// {n}* '(a+)(b+)?' matches once in 'aabcdef' but has two captures.
    #[structopt(short = '1', long = "first_capture_only")]
    pub first_capture_only: bool,
    /// Keep the full line if it does not match the pattern
    #[structopt(short = 'k', long)]
    pub keep_unmatched: bool,
    /// Maximum number of matching lines
    #[structopt(short = 'n', long)]
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
    use clap::IntoApp;
    GrabArgs::into_app().debug_assert()
}
