use ::std::path::PathBuf;

use ::clap::StructOpt;

#[derive(StructOpt, Debug, Clone)]
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
    //TODO @mverleg:
    /// Show generated Rust code, for debug purposes.
    #[structopt(long = "rsh-show-generated")]
    pub show_generated: bool,
    //TODO @mverleg:
    /// Extra arguments to pass to the Rust script.
    #[structopt()]
    pub args: Vec<String>,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    RshArgs::into_app().debug_assert()
}
