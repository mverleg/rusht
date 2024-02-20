use ::std::time::Duration;

use ::clap::Parser;
use ::parse_duration0::parse as parse_dur;

use crate::common::CommandArgs;

const KEY_DEFAULT: &'static str = "%{pwd}_%{env}_%{cmd}.cache";

#[derive(Parser, Debug, PartialEq)]
#[command(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired. Stderr is only shown on first run."
)]
pub struct CachedArgs {
    /// Duration the cache should be valid for, e.g. "30 min" or "1 day -1 hour".
    #[arg(value_parser = parse_dur, short = 'd', long = "duration", default_value = "15 min")]
    pub duration: Duration,
    /// Invalidates cache if the current git HEAD commit is different.
    #[arg(short = 'g', long)]
    pub git_head: bool,
    /// Invalidates cache if the current git branch merge-base commit is different.
    #[arg(short = 'b', long)]
    pub git_base: bool,
    /// Invalidates cache if the uncommitted git files change.
    #[arg(short = 'p', long)]
    pub git_pending: bool,
    #[arg(short = 'e', long)]
    pub env: Vec<String>,
    /// Just a text. Invalidates cache if a different text is passed.
    #[arg(short = 't', long)]
    pub text: Vec<String>,
    /// Does NOT invalidate cache if the command is run from a different directory.
    #[arg(short = 'D', long)]
    pub no_dir: bool,
    /// Does NOT cache if a different command is run. Should perhaps be used with e.g. --text
    #[arg(short = 'C', long)]
    pub no_command: bool,
    /// Does NOT cache if different env vars are passed to the command (does not include inherited env)
    #[arg(short = 'E', long)]
    pub no_direct_env: bool,
    #[arg(short = 's', long)]
    pub no_cached_output: bool,
    /// Use exit code 0 if the command is cached, and exit code 255 if it ran successfully.
    #[arg(short = 'x', long)]
    pub exit_code: bool,
    /// Print extra information, e.g. whether the command was run or not.
    #[arg(short = 'v', long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

impl CachedArgs {
    pub fn any_explicit_key(&self) -> bool {
        self.git_head || self.git_base || self.git_pending || !self.env.is_empty() || !self.text.is_empty()
    }
}

impl Default for CachedArgs {
    fn default() -> Self {
        CachedArgs {
            duration: Duration::from_secs(15 * 60),
            git_head: false,
            git_base: false,
            git_pending: false,
            env: vec![],
            text: vec![],
            no_dir: false,
            no_command: false,
            no_direct_env: false,
            no_cached_output: false,
            exit_code: false,
            verbose: false,
            cmd: CommandArgs::Cmd(Vec::new()),
        }
    }
}

#[test]
fn test_cli_args() {
    let mut args = CachedArgs::try_parse_from(&["cmd", "ls"]).unwrap();
    assert!(!args.any_explicit_key());
    assert_eq!(args, CachedArgs {
        cmd: CommandArgs::Cmd(vec!["ls".to_owned()]),
        ..CachedArgs::default()
    });
    args = CachedArgs::try_parse_from(&["cmd", "--duration", "1 year", "ls"]).unwrap();
    assert!(!args.any_explicit_key());
    args = CachedArgs::try_parse_from(&["cmd", "-d1y", "--git-head", "--git-pending", "ls", "-a", "-l", "-s", "-h"]).unwrap();
    assert!(args.any_explicit_key());
    args = CachedArgs::try_parse_from(&["cmd", "-d1y", "--text", "string", "ls", "-alsh"]).unwrap();
    assert!(args.any_explicit_key());
    args = CachedArgs::try_parse_from(&["cmd", "-d1y", "-gbpe", "ENV_VAR", "-CDEt", "string", "-t", "another string", "--", "ls"]).unwrap();
    assert!(args.any_explicit_key());
}
