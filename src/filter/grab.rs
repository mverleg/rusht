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
    //TODO @mverleg: case-insensitive
    #[structopt(help = "Regular expression to match. Returns the capture group if any, or the whole match otherwise.")]
    pub pattern: Regex,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    GrabArgs::into_app().debug_assert()
}

pub fn grab(
    args: GrabArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut consumer: impl FnMut(String),
) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run_grab(input: Vec<String>) -> Vec<String> {
        let args = GrabArgs {
            pattern: Regex::new("(a+)b").unwrap(),
        };
        let mut res = vec![];
        let mut lines = input.into_iter().map(|v| Ok(v));
        grab(
            args,
            || lines.next(),
            |line| res.push(line)).unwrap();
        res
    }

    #[test]
    fn test_empty() {
        let res = run_grab(vec![]);
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }
}