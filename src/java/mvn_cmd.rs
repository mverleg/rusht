use ::std::cmp::min;
use ::std::collections::HashMap;
use ::std::path::PathBuf;

use ::log::debug;
use ::smallvec::{smallvec, SmallVec};
use itertools::Itertools;

use crate::common::Task;
use crate::java::mvnw_args::TestMode;
use crate::java::newtype::Profile;

#[derive(Debug, PartialEq, Eq)]
pub struct MvnCmdConfig {
    /// Which modules to build. Empty means everything.
    pub modules: Vec<String>,
    pub tests: TestMode,
    pub verbose: bool,
    pub update: bool,
    pub clean: bool,
    pub install: bool,
    pub profiles: Vec<Profile>,
    pub threads: u32,
    pub max_memory_mb: u32,
    pub mvn_exe: PathBuf,
    pub mvn_arg: Vec<String>,
    pub java_home: PathBuf,
    pub cwd: PathBuf,
}

impl MvnCmdConfig {
    pub fn build_cmds(&self) -> SmallVec<[Task; 1]> {
        let single_cmd = self.modules.is_empty();

        let mut cmds = smallvec![];
        let mut args = vec![];
        if self.verbose {
            debug!("printing versions because of verbose mode");
            cmds.push(self.make_task(vec!["--version".to_owned()]));
        }

        // Clean
        if self.clean {
            if !single_cmd || !self.profiles.is_empty() {
                debug!("clean and build in separate commands, to clean everything while only building a subset (either because no --all or because of profiles)");
                let mut clean_args = vec!["clean".to_owned()];
                if !self.verbose {
                    clean_args.push("--quiet".to_owned());
                }
                cmds.push(self.make_task(clean_args));
            } else {
                debug!("clean and build in same command because of --all");
                args.push("clean".to_owned());
            }
        } else {
            debug!("not cleaning, incremental build");
        }

        // Determine maven stage
        let stage = if self.install {
            debug!("maven install requested");
            "install"
        } else if self.tests.run_any() && single_cmd {
            debug!("maven test because no install requested, there are tests, and the tests don't run in a separate command");
            "test"
        } else {
            debug!("maven compile because no install requested, and either there are no tests or they run in a separate command");
            "compile"
        };
        args.push(stage.to_owned());

        // Affected build modules
        if !self.modules.is_empty() {
            debug!(
                "building {} specific modules and their dependencies",
                self.modules.len()
            );
            for module in &self.modules {
                args.push("--projects".to_owned());
                args.push(format!(":{}", module));
            }
            args.push("--also-make".to_owned())
        } else {
            debug!("building all modules");
        }

        // Modifier flags
        args.push(format!("--threads={}", self.threads));
        if self.update {
            args.push("--update-snapshots".to_owned());
        } else {
            debug!("using offline mode because -U wasn't requested; try with -U if this fails");
            args.push("--offline".to_owned());
        }
        if !self.verbose {
            args.push("--quiet".to_owned());
        }

        // Default optimization flags
        self.add_opt_args(&mut args);

        // Tests
        match self.tests {
            TestMode::Files => {
                debug!("only running tests for changed files");
                unimplemented!()
            }
            TestMode::Modules => {
                debug!("running tests for all modules that have changed files");
                unimplemented!()
            }
            TestMode::All => {
                debug!("running all tests");
            }
            TestMode::NoRun => {
                debug!("building tests but not running them");
                args.push("-DskipTests".to_owned());
            }
            TestMode::NoBuild => {
                debug!("not building or running tests");
                args.push("-Dmaven.test.skip=true".to_owned());
                args.push("-DskipTests".to_owned());
            }
        }
        let mut test_task = None;
        if self.tests.run_any() && self.tests == TestMode::All {
            if single_cmd {
                debug!("build and test in same command (all modules are built)");
                self.add_test_args(&mut args);
            } else {
                debug!("build and test in separate commands, to build recursively but test only specific modules");
                let mut test_args = vec!["test".to_owned()];
                self.add_opt_args(&mut test_args);
                self.add_test_args(&mut test_args);
                test_task = Some(self.make_task(test_args));
            }
        }

        cmds.push(self.make_task(args));
        if let Some(tsk) = test_task {
            cmds.push(tsk);
        }

        cmds
    }

    fn make_task(&self, mut args: Vec<String>) -> Task {
        args.extend_from_slice(&self.mvn_arg);
        let mut extra_env = HashMap::new();
        extra_env.insert(
            "MAVEN_OPTS".to_owned(),
            format!(
                "-XX:+UseParallelGC -Xms{}m -Xmx{}m",
                min(256, self.max_memory_mb),
                self.max_memory_mb
            ),
        );
        extra_env.insert(
            "JAVA_HOME".to_owned(),
            self.java_home.to_str().unwrap().to_owned(),
        );
        if !self.profiles.is_empty() {
            debug!("(de)activating {} maven profiles", self.profiles.len());
            args.push(format!(
                "--activate-profiles='{}'",
                self.profiles.iter().join(",")
            ));
        }
        Task::new_with_env(
            self.mvn_exe.to_str().unwrap().to_owned(),
            args,
            self.cwd.to_owned(),
            extra_env,
        )
    }

    fn add_opt_args(&self, args: &mut Vec<String>) {
        args.push("-Djava.net.preferIPv4Stack=true".to_owned());
        args.push("-Dmanagedversions.skip=true".to_owned());
        args.push("-Dmanagedversions.failOnError=false".to_owned());
        args.push("-Denforcer.skip=true".to_owned());
        args.push("-Ddatabase.skip=true".to_owned());
        args.push("-Dmaven.javadoc.skip=true".to_owned());
    }

    fn add_test_args(&self, args: &mut Vec<String>) {
        args.push("-DskipITs".to_owned());
        args.push("-Dsurefire.printSummary=false".to_owned());
        args.push("-DfailIfNoTests=false".to_owned());
        args.push("-Dparallel=all".to_owned());
        args.push("-DperCoreThreadCount=false".to_owned());
        args.push(format!(
            "-DthreadCount={}",
            if self.threads > 1 {
                4 * self.threads
            } else {
                1
            }
        ));
    }
}
