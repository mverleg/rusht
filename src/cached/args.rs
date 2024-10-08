use ::std::time::Duration;

use ::clap::Parser;
use ::parse_duration0::parse as parse_dur;

use crate::common::CommandArgs;

#[derive(Parser, Debug, PartialEq)]
#[command(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired. Stderr is only shown on first run."
)]
pub struct CachedArgs {
    /// Duration after which the cache should be invalidated, e.g. "30 min" or "1 day -1 hour".
    #[arg(value_parser = parse_dur, short = 'd', long = "duration", default_value = "15 min")]
    pub duration: Duration,
    #[clap(flatten)]
    pub key: CachedKeyArgs,
    /// When loading from cache, do not show the previous output.
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

#[derive(Parser, Debug, PartialEq, Default)]
#[command()]
pub struct CachedKeyArgs {
    /// Invalidates cache if the current git HEAD commit is different. For only code changes, see --git-head-diff
    #[arg(short = 'g', long)]
    pub git_head: bool,
    /// Invalidates cache if the current git branch merge-base commit is different.
    #[arg(short = 'b', long, conflicts_with = "git_head")]
    pub git_base: bool,
    /// Invalidates cache if the diff of HEAD has changed. This is more lenient than --git-head as it ignores commit message and trivial rebases.
    #[arg(short = 'G', long, conflicts_with = "git_head")]
    pub git_head_diff: bool,
    /// Invalidates cache if we're in a different git repo. This doesn't make much sense without --no-dir.
    #[arg(long)]
    pub git_repo_dir: bool,
    /// Invalidates cache if the uncommitted git files change.
    #[arg(short = 'p', long)]
    pub git_pending: bool,
    /// Name of an environment variable. Invalidates cache if the value changes.
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
}


impl CachedArgs {
    pub fn any_explicit_key(&self) -> bool {
        self.key.git_head || self.key.git_head_diff || self.key.git_base || self.key.git_repo_dir ||
            self.key.git_pending || !self.key.env.is_empty() || !self.key.text.is_empty()
    }
}

impl Default for CachedArgs {
    fn default() -> Self {
        CachedArgs {
            duration: Duration::from_secs(15 * 60),
            key: CachedKeyArgs {
                git_head: false,
                git_base: false,
                git_head_diff: false,
                git_repo_dir: false,
                git_pending: false,
                env: vec![],
                text: vec![],
                no_dir: false,
                no_command: false,
                no_direct_env: false,
            },
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
    args = CachedArgs::try_parse_from(&["cmd", "-d1y", "-gpe", "ENV_VAR", "-CDEt", "string", "-t", "another string", "--", "ls"]).unwrap();
    assert!(args.any_explicit_key());
}
