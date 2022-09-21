use ::std::fmt;
use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::StructOpt;
use ::clap::ValueEnum;
use ::regex::Regex;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "rsh", about = "Compile and run a Rust snippet.")]
pub struct RshArgs {
    /// Name of the Rust script to run.
    #[structopt()]
    pub script: PathBuf,
    /// Build the code, but do not execute it.
    #[structopt(short = 'b', long = "build-only")]
    pub build_only: bool,
    /// Show generated Rust code, for debug purposes.
    #[structopt(long = "show-generated")]
    pub show_generated: bool,
    /// Extra arguments to pass to the Rust script.
    #[structopt(subcommand)]
    pub args: ExtraArgs,
}
//TODO @mverleg:

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum ExtraArgs {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    RshArgs::into_app().debug_assert()
}
