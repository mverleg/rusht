use ::clap::Parser;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "piped",
    about = "Split into two commands, and pipe the output of the first into the second."
)]
pub struct PipedArgs {
    /// Which token separates the two commands. Only the first occurrence is matched.
    #[arg(short = 's', long = "separator", default_value = "//")]
    pub separator: String,
    /// Pipe stderr instead of stdout into the next command.
    #[arg(short = 'e', long = "stderr")]
    pub stderr: bool,
    /// Number of lines to buffer between the commands.
    #[arg(long = "pipe-buffer-size", default_value = "4", value_parser = parse_buffer_size, hide_short_help = true)]
    pub pipe_buffer_size: u32,
    #[command(subcommand)]
    pub cmds: CommandArgs,
}
//TODO @mverleg: 1-to-1, 1-to-many

#[test]
fn test_cli_args() {
    PipedArgs::try_parse_from(&["cmd", "-s=//", "ls", "//", "wc", "-l"]).unwrap();
}


fn parse_buffer_size(txt: &str) -> Result<u32, String> {
    match txt.parse::<u32>() {
        Ok(nr) => {
            if nr < 2 {
                return Err("must be at least 2".to_owned());
            }
            Ok(nr)
        }
        Err(err) => Err(format!("could not parse argument, err '{}'", err)),
    }
}
