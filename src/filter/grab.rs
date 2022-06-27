use ::std::io;

use ::clap::StructOpt;
use ::regex::Captures;
use ::regex::Regex;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grab",
    about = "Filter lines by regular expression, keeping only the matching capture group."
)]
pub struct GrabArgs {
    //TODO @mverleg: keep unmatching lines
    //TODO @mverleg: able to match multiple times per line
    #[structopt(help = "Regular expression to match. Returns the capture group if any, or the whole match otherwise.")]
    pub pattern: Regex,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    GrabArgs::into_app().debug_assert()
}

pub fn grab(args: GrabArgs, mut consumer: impl FnMut(String), mut line_supplier: fn() -> Option<io::Result<String>>) -> Result<(), String> {
    while let Some(line_res) = line_supplier() {
        let line = match line_res {
            Ok(line) => line,
            Err(err) => return Err(format!("failed to read line: {}", err)),
        };
        match args.pattern.captures(&line) {
            Some(captures) => {
                consumer(line)
            }
            None => {}
        }
    }
    Ok(())
}
