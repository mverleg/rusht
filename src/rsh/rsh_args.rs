use ::std::path::PathBuf;

use ::clap::Parser;

#[derive(Parser, Debug, Clone)]
#[structopt(
    name = "rsh",
    about = "Compile and run a Rust snippet.",
    after_help = "If you see this instead of your command's help, try adding -- before arguments."
)]
pub struct RshArgs {
    /// Name of the Rust script to run.
    #[structopt()]
    pub script: PathBuf,
    /// Rebuild the code independent of cache. Run unless --rsh-build-only.
    #[structopt(long = "rsh-rebuild")]
    pub force_rebuild: bool,
    /// Build the code, but do not execute it.
    #[structopt(long = "rsh-build-only")]
    pub build_only: bool,
    /// Show generated Rust code, for debug purposes.
    #[structopt(long = "rsh-keep-generated")]
    pub keep_generated: bool,
    //TODO @mverleg:
    /// Extra arguments to pass to the Rust script.
    #[structopt()]
    pub args: Vec<String>,
}

#[test]
fn test_cli_args() {
use ::clap::FromArgMatches;
    RshArgs::from_arg_matches().unwrap();
}
