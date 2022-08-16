use ::std::fmt::{Display, Formatter};
use ::std::str::FromStr;

use ::clap::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "java",
    about = "Wrapper for maven (daemon) to add speed flags. Needs maven and git."
)]
pub struct MvnwArgs {
    #[structopt(
        short = 'c',
        long,
    )]
    /// Do a clean build (also cleans unaffected modules).
    pub clean: bool,
    #[structopt(
        short = 'i',
        long,
    )]
    /// Install the modules into local .m2 after building them.
    pub install: bool,
    #[structopt(
        short = 'a',
        long,
    )]
    /// Build all the code, not just affected files.
    pub all: bool,
    #[structopt(
        short = 'U',
        long,
    )]
    /// Update snapshots, even if it was recently done.
    pub update: bool,
    #[structopt(short = 't', long)]
    /// Run tests in affected modules.
    pub tests: bool,
    #[structopt(
        short = 'p',
        long,
        conflicts_with = "tests"
    )]
    /// Only build prod (main) code, skip building tests.
    pub prod_only: bool,
    #[structopt(
        short = 'v',
        long,
    )]
    /// Show the maven commands being run, and the build output.
    pub verbose: bool,
    #[structopt(
        short = 'V',
        long,
    )]
    /// Only show the maven commands to be ran, do not actually run them.
    pub show_cmds_only: bool,
    #[structopt(
        short = 'x',
        long = "affected",
        default_value = "any-change",
        conflicts_with = "all"
    )]
    /// How to determine which files/modules have been affected: [a]ny-change / [r]ecent / [u]ncommitted / [h]ead / [b]ranch.
    ///
    /// [u]ncommitted: uncommitted changes (staged or otherwise)
    /// [h]ead: changes from the head commit
    /// [b]ranch: changes from any commit in the branch, that aren't in origin/master (or main)
    /// [a]ny-change: uncommitted + branch
    /// [r]ecent: head + branch
    pub affected_policy: AffectedPolicy,
    #[structopt(
        long,
    )]
    /// Number of threads to build with. Defaults to number of cores. Multiplied by 4 for running tests.
    pub threads: Option<u32>,
    #[structopt(
        long = "max-memory",
        default_value = "8192"
    )]
    /// Maximum memory to build, in MB.
    pub max_memory_mb: u32,
    #[structopt(
        long,
        default_value = "mvn"
    )]
    /// Maven executable. Can be used to select a different path or switch to mvnd.
    pub mvn_exe: String,
    #[structopt(long)]
    /// Extra arguments to pass to maven.
    pub mvn_arg: Vec<String>,
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
    Head,
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
            "h" | "head" => AffectedPolicy::Head,
            "b" | "branch" => AffectedPolicy::Branch,
            other => return Err(format!("unknown affected files policy: {}", other)),
        })
    }
}

impl Display for AffectedPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AffectedPolicy::AnyChange => "any-change",
                AffectedPolicy::Recent => "recent",
                AffectedPolicy::Uncommitted => "uncommitted",
                AffectedPolicy::Head => "head",
                AffectedPolicy::Branch => "branch",
            }
        )
    }
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    MvnwArgs::into_app().debug_assert()
}
