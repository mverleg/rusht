use ::std::path::PathBuf;

use ::clap::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "rsh", about = "Compile and run a Rust snippet.")]
pub struct RshArgs {
    /// Name of the Rust script to run.
    #[structopt()]
    pub script: PathBuf,
    /// Build the code, but do not execute it.
    #[structopt(long = "rsh-build-only")]
    pub build_only: bool,
    /// Show generated Rust code, for debug purposes.
    #[structopt(long = "rsh-show-generated")]
    pub show_generated: bool,
    /// Extra arguments to pass to the Rust script.
    #[structopt()]
    pub args: Vec<String>,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    RshArgs::into_app().debug_assert()
}
