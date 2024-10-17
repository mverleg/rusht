use ::clap::Parser;
use crate::ExitStatus;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "server",
    about = "Run a simple http server that can run other rusht commands, for use through e.g. curl.",
    after_help = "Example: curl -L --no-progress-meter --fail -X POST --data 'namesafe -i \"a+1\" -x=n' 'rusht.example:8008'"
)]
pub struct ServerArgs {
    /// Port to listen on
    #[arg(default_value = "8008")]
    pub port: u16,
}

#[test]
fn test_cli_args() {
    ServerArgs::try_parse_from(&["cmd", "-p", "8080"]).unwrap();
}

#[derive(Debug)]
enum OutFmt { Text, Json }

//TODO @mverleg: should also have an easier way to just pass a command only, maybe as url? does curl do urlencoding automatically?
#[derive(Debug)]
struct Input {
    cmd: Vec<String>,
    //TODO @mverleg: default: json
    fmt: Option<OutFmt>,
    stdin: Option<String>,
}

/// When format is text, the output is stdout if status==0, and stderr otherwise, with http code 4xx/5xx
//  ^ TODO @mverleg: https://docs.rs/actix-web/latest/actix_web/enum.Either.html
#[derive(Debug)]
struct Output {
    output: String,
    stderr: String,
    status: u16,
    duration_ms: u64,
}

pub fn serve(args: ServerArgs) -> ExitStatus {
    todo!("not implemented yet");  //TODO @mverleg:
    ExitStatus::ok()
}
