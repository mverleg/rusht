use ::std::path::PathBuf;

use ::clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rsh",
    about = "Compile and run a Rust snippet.",
    after_help = "If you see this instead of your command's help, try adding -- before arguments."
)]
pub struct RshArgs {
    /// Name of the Rust script to run.
    #[arg()]
    pub script: PathBuf,
    /// Rebuild the code independent of cache. Run unless --rsh-build-only.
    #[arg(long = "rsh-rebuild")]
    pub force_rebuild: bool,
    /// Build the code, but do not execute it.
    #[arg(long = "rsh-build-only")]
    pub build_only: bool,
    /// Show generated Rust code, for debug purposes.
    #[arg(long = "rsh-keep-generated")]
    pub keep_generated: bool,
    //TODO @mverleg:
    /// Extra arguments to pass to the Rust script.
    #[arg()]
    pub args: Vec<String>,
}

#[test]
fn test_cli_args() {
    RshArgs::try_parse_from(&["cmd", "--help"]).unwrap();
}
