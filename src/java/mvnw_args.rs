use ::std::str::FromStr;

use ::clap::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "java",
    about = "Wrapper for maven (daemon) to add speed flags. Needs maven and git."
)]
pub struct MvnwArgs {
    #[structopt(short = 'c', long, help = "Do a clean build (also cleans unaffected modules).")]
    pub clean: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'a', long, help = "Build all the code, not just affected files.")]
    pub all: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'U', long, help = "Update snapshots, even if it was recently done.")]
    pub update: bool,
    //TODO @mverleg: ^
    #[structopt(short = 't', long, help = "Run tests in affected modules.")]
    pub affected_tests: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'T', long, help = "Run tests in all modules. Implies --all.", conflicts_with = "affected-tests")]
    pub all_tests: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'p', long, help = "Only build prod (main) code, skip building tests.", conflicts_with = "affected-tests", conflicts_with = "all-tests")]
    pub prod_only: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'v', long, help = "Show the maven commands being run, and the build output.")]
    pub verbose: bool,
    //TODO @mverleg: ^
    #[structopt(short = 'V', long, help = "Only show the maven commands to be ran, do not actually run them.")]
    pub show_cmds_only: bool,
    //TODO @mverleg: ^
    #[structopt(
        short = 'x',
        long = "affected",
        help = "How to determine which files/modules have been affected: [a]ny-change / [r]ecent / [u]ncommitted / [c]ommit / [b]ranch.",
        conflicts_with = "all",
    )]
    pub affected_policy: AffectedPolicy,
    //TODO @mverleg: ^
}
//TODO @mverleg: pass extra maven args directly
//TODO @mverleg: also include linting?

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AffectedPolicy {
    /// `Branch` + `Uncommmitted`
    AnyChange,
    #[default]
    /// `Commit` + `Uncommmitted`
    Recent,
    /// All uncommitted changes.
    Uncommitted,
    /// All changes in the head commit.
    Commit,
    /// All commits in the branch (that are not in master).
    Branch,
}

impl FromStr for AffectedPolicy {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text.to_lowercase().as_str() {
            "a" | "any-change" | "all" => AffectedPolicy::AnyChange,
            "r" | "recent" => AffectedPolicy::Recent,
            "u" | "uncommitted" => AffectedPolicy::Uncommitted,
            "c" | "commit" => AffectedPolicy::Commit,
            "b" | "branch" => AffectedPolicy::Branch,
            other => return Err(format!("unknown affected files policy: {}", other)),
        })
    }
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    MvnwArgs::into_app().debug_assert()
}
