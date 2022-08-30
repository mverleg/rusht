use ::clap::StructOpt;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "piped",
    about = "Split into two commands, and pipe the output of the first into the second."
)]
pub struct PipedArgs {
    #[structopt(short = 's', long = "separator", default_value = "//")]
    pub separator: bool,
    #[structopt(subcommand)]
    pub cmds: CommandArgs,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    PipedArgs::into_app().debug_assert()
}
