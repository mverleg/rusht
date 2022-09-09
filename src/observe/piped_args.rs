use ::clap::StructOpt;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "piped",
    about = "Split into two commands, and pipe the output of the first into the second."
)]
pub struct PipedArgs {
    /// Which token separates the two commands. Only the first occurrence is matched.
    #[structopt(short = 's', long = "separator", default_value = "//")]
    pub separator: String,
    /// Pipe stderr instead of stdout into the next command.
    #[structopt(short = 'e', long = "stderr")]
    pub stderr: bool,
    #[structopt(subcommand)]
    pub cmds: CommandArgs,
}
//TODO @mverleg: 1-to-1, 1-to-many

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    PipedArgs::into_app().debug_assert()
}
