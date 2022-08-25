use ::std::cmp::min;
use ::std::collections::HashMap;
use ::std::path::PathBuf;
use std::collections::HashSet;

use ::itertools::Itertools;
use ::log::debug;
use ::log::warn;
use ::smallvec::SmallVec;

use crate::common::Task;
use crate::java::mvnw_args::TestMode;
use crate::java::newtype::Profile;

#[derive(Debug, PartialEq, Eq)]
pub struct MvnCmdConfig {
    /// Which files were changed. Might have been deleted.
    pub changed_files: HashSet<PathBuf>,
    /// Which modules to build. Empty means everything.
    pub modules: Vec<String>,
    pub tests: TestMode,
    pub lint: bool,
    pub checkstyle_version: String,
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

#[derive(Debug, Default)]
struct MvnTasks {
    version: Option<Task>,
    clean: Option<Task>,
    install_lint: Option<Task>,
    lint: Option<Task>,
    build: Option<Task>,
    test: Option<Task>,
}

impl MvnCmdConfig {
    /// Return commands that can be started concurrently and will wait for eachother.
    pub fn build_cmds(&self) -> SmallVec<[Task; 1]> {
        self.collect_tasks().flatten()
    }

    fn collect_tasks(&self) -> MvnTasks {
        let single_cmd = self.modules.is_empty();

        let mut tasks = MvnTasks::default();
        let mut args = vec![];
        if self.verbose {
            debug!("printing versions because of verbose mode");
            tasks.version = Some(self.make_mvn_task(vec!["--version".to_owned()]));
        }

        // Clean
        if self.clean {
            if !single_cmd || !self.profiles.is_empty() {
                debug!("clean and build in separate commands, to clean everything while only building a subset (either because no --all or because of profiles)");
                let mut clean_args = vec!["clean".to_owned()];
                if !self.verbose {
                    clean_args.push("--quiet".to_owned());
                }
                tasks.clean = Some(self.make_mvn_task(clean_args));
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
        } else if self.tests == TestMode::NoBuild {
            debug!("maven compile because no install or tests requested");
            "compile"
        } else {
            debug!("maven test-compile because no install requested, and tests are run in a separate command");
            "test-compile"
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

        // Lint
        if !self.lint {
            debug!("no lint requested, skipping checkstyle");
        } else if self.changed_files.is_empty() {
            debug!("no affected files, checkstyle lint was requested but will be skipped");
        } else {
            let mut checkstyle_conf_pth = self.cwd.clone();
            checkstyle_conf_pth.push("sputnik-rules");
            checkstyle_conf_pth.push("checkstyle.xml");
            if checkstyle_conf_pth.is_file() {
                debug!(
                    "linting enabled, found checkstyle config at: {}",
                    checkstyle_conf_pth.to_string_lossy()
                );
                let (task, checkstyle_jar_pth) =
                    ensure_checkstyle_jar_exists(&self.checkstyle_version);
                if let Some(task) = task {
                    tasks.install_lint = Some(task);
                }
                let mut lint_args = vec![
                    format!("-Xmx{}m", self.max_memory_mb),
                    "-jar".to_owned(),
                    checkstyle_jar_pth.to_str().unwrap().to_owned(),
                    "-c".to_owned(),
                    checkstyle_conf_pth.to_str().unwrap().to_owned(),
                ];
                lint_args.extend_from_slice(
                    &self
                        .changed_files
                        .iter()
                        .map(|af| {
                            af.to_str()
                                .expect("changed file path not unicode")
                                .to_owned()
                        })
                        .collect::<Vec<_>>(),
                );
                tasks.lint = Some(Task::new("java".to_owned(), lint_args, self.cwd.clone()));
            } else {
                warn!(
                    "skipping checkstyle because config was not found at '{}'",
                    checkstyle_conf_pth.to_string_lossy()
                );
            }
        }

        // Tests
        match self.tests {
            TestMode::Files => {
                debug!("only running tests for changed files");
                unimplemented!("test mode files not implemented")
            }
            TestMode::Modules => {
                debug!("running tests for all modules that have changed files");
                unimplemented!("test mode modules not implemented")
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
                test_task = Some(self.make_mvn_task(test_args));
            }
        }

        tasks.build = Some(self.make_mvn_task(args));
        if let Some(test_tsk) = test_task {
            tasks.test = Some(test_tsk);
        }

        tasks
    }

    fn make_mvn_task(&self, args: Vec<String>) -> Task {
        let mut extra_env = HashMap::new();
        extra_env.insert(
            "MAVEN_OPTS".to_owned(),
            format!(
                "-XX:+UseG1GC -Xms{}m -Xmx{}m",
                min(256, self.max_memory_mb),
                self.max_memory_mb
            ),
        );
        self.make_task(self.mvn_exe.to_str().unwrap(), args, extra_env)
    }

    fn make_task(
        &self,
        exe: impl Into<String>,
        mut args: Vec<String>,
        mut extra_env: HashMap<String, String>,
    ) -> Task {
        args.extend_from_slice(&self.mvn_arg);
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
        Task::new_with_env(exe.into(), args, self.cwd.to_owned(), extra_env)
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

fn ensure_checkstyle_jar_exists(version: &str) -> (Option<Task>, PathBuf) {
    let cache_dir = dirs::cache_dir().expect("failed to find cache directory");
    let mut checkstyle_jar_pth = cache_dir.clone();
    checkstyle_jar_pth.push(format!("checkstyle-{}.jar", version));
    if checkstyle_jar_pth.is_file() {
        debug!(
            "found checkstyle jar at: {}",
            checkstyle_jar_pth.to_string_lossy()
        );
        return (None, checkstyle_jar_pth);
    }
    let task = Task::new(
        "curl".to_owned(),
        vec![
            "-L".to_owned(),
            format!("https://github.com/checkstyle/checkstyle/releases/download/checkstyle-8.1/checkstyle-{}-all.jar", version),
            "--silent".to_owned(),
            "--output".to_owned(),
            checkstyle_jar_pth.to_str().unwrap().to_owned(),
        ], cache_dir
    );
    debug!(
        "creating task to download checkstyle jar: {}",
        task.as_str()
    );
    (Some(task), checkstyle_jar_pth)
}

impl MvnTasks {
    fn flatten(self) -> SmallVec<[Task; 1]> {
        let MvnTasks { version, clean, install_lint, lint, build, test } = self;
        unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    }
}