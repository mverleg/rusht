use ::std::fmt;
use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::Parser;
use ::clap::ValueEnum;
use ::regex::Regex;

use crate::java::newtype::{FullyQualifiedName, Profile};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "java",
    about = "Wrapper for maven (daemon) to add speed flags. Needs maven and uses git.",
    after_help = "Thanks for using! Note: some options are only visible with --help (not with -h).",
    group = clap::ArgGroup::new("test").multiple(false),
)]
pub struct MvnwArgs {
    /// Do a clean build (also cleans unchanged modules).
    #[arg(short = 'c', long)]
    pub clean: bool,
    /// Install the modules into local .m2 after building them.
    #[arg(short = 'i', long)]
    pub install: bool,
    /// Build all the code, not just changed files.
    #[arg(short = 'a', long)]
    pub all: bool,
    /// Update snapshots, even if it was recently done.
    #[arg(short = 'U', long)]
    pub update: bool,
    /// Execute these java classes.
    ///
    /// * Something like `com.company.package.Main`.
    /// {n}* Should contain `public static void main(String[] args)`.
    /// {n}* Class must be inside selected module, which may be controlled by --affected.
    /// {n}* Must be in selected profile, if any, and mvn exec plugin must be available.
    /// {n}* This does not automatically disable running unit tests.
    #[arg(short = 'm', long = "exec-main")]
    pub execs: Vec<FullyQualifiedName>,

    /// Run tests that were changed, or that match files that were changed (i.e. XyzTest if Xyz is changed). Default.
    #[arg(long = "test-files", group = "test")]
    test_files: bool,
    /// All tests in modules that contain changes.
    #[arg(short = 't', long = "test-modules", group = "test")]
    test_modules: bool,
    /// Run all the tests.
    #[arg(long = "test-all", group = "test")]
    test_all: bool,
    /// Do not run any tests (but still build them).
    #[arg(long = "test-none", group = "test")]
    test_none: bool,
    /// Only build prod (main) code, skip building tests.
    #[arg(short = 'T', long = "prod-only", group = "test")]
    prod_only: bool,

    /// Do not run checkstyle lints on the code.
    #[arg(short = 'L', long = "no-lint")]
    pub no_lint: bool,

    /// Show the maven version, and the output of commands.
    #[arg(short = 'v', long)]
    pub verbose: bool,
    /// Only show the maven commands to be ran, do not actually run them.
    #[arg(short = 'V', long, hide_short_help = true)]
    pub show_cmds_only: bool,
    /// How to determine which files/modules have been changed.
    ///
    /// [u]ncommitted: uncommitted changes (staged or otherwise)
    /// {n}[h]ead: changes from the head commit
    /// {n}[b]ranch: changes from any commit in the branch, that aren't in origin/master (or main)
    /// {n}[a]ny-change: uncommitted + branch
    /// {n}[r]ecent: uncommitted + head
    #[arg(
        value_enum,
        short = 'x',
        long = "affected",
        default_value = "recent",
        conflicts_with = "all"
    )]
    pub affected_policy: AffectedPolicy,
    /// Number of threads to build with. Defaults to number of cores. Multiplied by 4 for running tests.
    #[arg(long, value_parser = strictly_positive, hide_short_help = true)]
    pub threads: Option<u32>,
    /// Maximum memory to build, in MB.
    #[arg(long = "max-memory", value_parser = strictly_positive, default_value = "8192", hide_short_help = true)]
    pub max_memory_mb: u32,
    /// Maximum memory to execute, in MB. Default to same as build.
    #[arg(long = "max-exec-memory", value_parser = strictly_positive, hide_short_help = true)]
    pub max_exec_memory_mb: Option<u32>,
    /// Maven executable. Can be used to select a different path or switch to mvnd.
    #[arg(long, default_value = "mvn", hide_short_help = true)]
    pub mvn_exe: PathBuf,
    /// Extra arguments to pass on to maven, e.g. -a=-Djava.net.preferIPv4Stack=true
    #[arg(short = 'a', long = "mvn-arg")]
    pub mvn_args: Vec<String>,
    /// Maven profiles to activate. Prefix '!' to deactivate.
    #[arg(short = 'P', long = "profile")]
    pub profiles: Vec<Profile>,
    /// Maven projects to build. Defaults to current working directory.
    #[arg(long = "proj-root", hide_short_help = true)]
    pub proj_roots: Vec<PathBuf>,

    /// Re-run the commands with --clean --update if the output matches this pattern
    #[arg(short = 'C', long = "rebuild-if-match", hide_short_help = true)]
    pub rebuild_if_match: Vec<Regex>,
    //TODO @mverleg: ^
    /// Fail the command if the newly added code matches the regex.
    #[arg(short = 'g', long = "fail-if-added", hide_short_help = true)]
    pub fail_if_added: Vec<Regex>,
    //TODO @mverleg: ^
}

#[test]
fn test_cli_args() {
    MvnwArgs::try_parse_from(&[
        "cmd",
        "--all",
        "--test-none",
        "--max-exec-memory=2048",
        "--fail-if-added=//TODO @mverleg:",
        "--fail-if-added=^import .*\\.shaded\\.",
        "--rebuild-if-match=error: Javadoc retention failed; ",
        "--install",
        "--clean",
        "--update",
    ])
    .unwrap();
}

fn strictly_positive(val: &str) -> Result<u32, String> {
    match val.parse::<u32>() {
        Ok(nr) => {
            if nr >= 1 {
                Ok(nr)
            } else {
                Err("must be at least 1".to_owned())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffectedPolicy {
    /// `Branch` + `Uncommmitted`
    AnyChange,
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
            other => return Err(format!("unknown changed files policy: {}", other)),
        })
    }
}

impl fmt::Display for AffectedPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    #[default]
    Files,
    Modules,
    All,
    NoRun,
    NoBuild,
}

impl TestMode {
    pub fn run_any(&self) -> bool {
        *self != TestMode::NoRun && *self != TestMode::NoBuild
    }
}

impl MvnwArgs {
    pub fn test(&self) -> TestMode {
        match (
            self.test_files,
            self.test_modules,
            self.test_all,
            self.test_none,
            self.prod_only,
        ) {
            (false, false, false, false, false) => TestMode::NoRun,  // revert to Files
            (true, false, false, false, false) => TestMode::Files,
            (false, true, false, false, false) => TestMode::Modules,
            (false, false, true, false, false) => TestMode::All,
            (false, false, false, true, false) => TestMode::NoRun,
            (false, false, false, false, true) => TestMode::NoBuild,
            _ => unreachable!("mutually exclusive arguments provided, CLI should prevent this"),
        }
    }

    pub fn is_test_arg_provided(&self) -> bool {
        !matches!(
            (
                self.test_files,
                self.test_modules,
                self.test_all,
                self.test_none,
            ),
            (false, false, false, false)
        )
    }
}
