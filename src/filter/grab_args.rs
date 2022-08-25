use ::clap::StructOpt;
use ::regex::Regex;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grab",
    about = "Filter lines by regular expression, keeping only the matching capture group."
)]
pub struct GrabArgs {
    #[structopt()]
    /// Regular expression to match. Returns the capture group if any, or the whole match otherwise.
    pub pattern: Regex,
    #[structopt(short = '1', long)]
    /// Only print the first capture group, even if there are multiple
    pub first_only: bool,
    #[structopt(short = 'k', long)]
    /// Keep the full line if it does not match the pattern
    pub keep_unmatched: bool,
    #[structopt(short = 'n', long)]
    /// Maximum number of matching lines
    pub max_lines: Option<u32>,
}

impl Default for GrabArgs {
    fn default() -> Self {
        GrabArgs {
            pattern: Regex::new(".*").unwrap(),
            first_only: false,
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
