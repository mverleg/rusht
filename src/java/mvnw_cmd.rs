use ::std::cmp::min;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::path::PathBuf;
use std::cmp::max;

use ::itertools::Itertools;
use ::log::debug;
use ::log::warn;
use ::smallvec::SmallVec;

use crate::common::Dependent;
use crate::common::Task;
use crate::java::mvnw_args::TestMode;
use crate::java::newtype::{FullyQualifiedName, Profile};

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
    pub phase_override: Option<String>,
    pub execs: Vec<FullyQualifiedName>,
    pub profiles: Vec<Profile>,
    pub threads: u32,
    pub max_memory_mb: u32,
    pub max_exec_memory_mb: u32,
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
    exes: SmallVec<[Task; 1]>,
}

impl MvnCmdConfig {
    /// Return commands that can be started concurrently and will wait for eachother.
    pub fn build_cmds(&self) -> Vec<Dependent> {
        self.collect_tasks().flatten()
    }

    fn collect_tasks(&self) -> MvnTasks {
        let single_cmd = self.modules.is_empty() && self.profiles.is_empty();

        let mut tasks = MvnTasks::default();
        let mut args = vec![];
        if self.verbose {
            debug!("printing versions because of verbose mode");
            tasks.version = Some(self.make_mvn_task(vec!["--version".to_owned()]));
        }

        // Clean
        if self.clean {
            if !single_cmd {
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

        // Lint
        if !self.lint {
            debug!("no lint requested, skipping checkstyle");
        } else if self.changed_files.is_empty() {
            debug!("no affected files, checkstyle lint was requested but will be skipped");
        } else {
            //TODO @mverleg: avoid doing this if all files are deleted
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
                tasks.lint = Some(Task::new(
                    "java".to_owned(),
                    lint_args,
                    self.cwd.clone(),
                    None,
                ));
            } else {
                warn!(
                    "skipping checkstyle because config was not found at '{}'",
                    checkstyle_conf_pth.to_string_lossy()
                );
            }
        }

        // Determine maven stage
        let stage = if let Some(phase) = &self.phase_override {
            debug!("maven phase '{phase}' was explicitly requested");
            phase
        } else if self.tests.run_any() && single_cmd {
            debug!("maven verify because no install requested, there are tests (that might be ITs), and the tests don't run in a separate command");
            "integration-test"
        } else if self.tests == TestMode::NoBuild {
            debug!("maven compile because no install or tests requested");
            "compile"
        } else {
            debug!("maven test-compile because no install requested, and tests are run in a separate command");
            //"test-compile"
            //TODO @mverleg: using package because goat does not compile well without it
            "package"
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
        args.push(format!("-Dmaven.artifact.threads={}", max(8, 4 * self.threads)));
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
                if single_cmd {
                    args.push("-DskipTests".to_owned());
                }
            }
            TestMode::NoBuild => {
                debug!("not building or running tests");
                if single_cmd {
                    args.push("-Dmaven.test.skip=true".to_owned());
                    args.push("-DskipTests".to_owned());
                }
            }
        }
        if !single_cmd || !self.tests.run_any() {
            debug!("skipping tests in build command, since tests are run in a separate command, or not at all");
            args.push("-DskipTests=true".to_owned());
        }
        if self.tests.run_any() {
            if single_cmd {
                debug!("build and test in same command (all modules are built)");
                self.add_test_args(&mut args);
            } else {
                debug!("build and test in separate commands, to build recursively but test only specific modules");
                let mut test_args = vec!["test".to_owned()];
                self.add_opt_args(&mut test_args);
                self.add_test_args(&mut test_args);
                tasks.test = Some(self.make_mvn_task(test_args));
            }
        }

        tasks.build = Some(self.make_mvn_task(args));

        // Execute a class.
        for exec in &self.execs {
            let mut exe_args = vec![
                "exec:java".to_owned(),
                format!("-Dexec.mainClass=\"{}\"", exec),
            ];
            self.add_opt_args(&mut exe_args);
            tasks
                .exes
                .push(self.make_mvn_task_with_mem(exe_args, self.max_exec_memory_mb));
        }

        tasks
    }

    fn make_mvn_task_with_mem(&self, args: Vec<String>, memory_mb: u32) -> Task {
        let mut extra_env = HashMap::new();
        extra_env.insert(
            "MAVEN_OPTS".to_owned(),
            format!(
                "-XX:+UseG1GC -Xms{}m -Xmx{}m",
                min(256, memory_mb),
                memory_mb
            ),
        );
        self.make_task(self.mvn_exe.to_str().unwrap(), args, extra_env)
    }

    fn make_mvn_task(&self, args: Vec<String>) -> Task {
        self.make_mvn_task_with_mem(args, self.max_memory_mb)
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
                "--activate-profiles=\"{}\"",
                self.profiles.iter().join(",")
            ));
        }
        Task::new_with_env(exe.into(), args, self.cwd.to_owned(), None, extra_env)
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
            format!("https://github.com/checkstyle/checkstyle/releases/download/checkstyle-{version}/checkstyle-{version}-all.jar"),
            "--silent".to_owned(),
            "--show-error".to_owned(),
            "--fail".to_owned(),
            "--output".to_owned(),
            checkstyle_jar_pth.to_str().unwrap().to_owned(),
        ],
        cache_dir,
        None
    );
    debug!(
        "creating task to download checkstyle jar: {}",
        task.as_str()
    );
    (Some(task), checkstyle_jar_pth)
}

impl MvnTasks {
    fn flatten(self) -> Vec<Dependent> {
        let MvnTasks {
            version,
            clean,
            install_lint,
            lint,
            build,
            test,
            exes,
        } = self;
        let version = Dependent::new_optional("version", version);
        let mut clean = Dependent::new_optional("clean", clean);
        clean.depends_on(&version);
        let mut install_lint = Dependent::new_optional("install_lint", install_lint);
        install_lint.depends_on(&version);
        let mut lint = Dependent::new_optional("lint", lint);
        lint.depends_on(&install_lint);
        let mut build = Dependent::new_named("build", build.expect("build task must always exist"));
        build.depends_on(&lint); // linter sometimes fails on @Immutables if build is running
        build.depends_on(&clean);
        let mut test = Dependent::new_optional("test", test);
        test.depends_on(&build);
        let exes = exes
            .into_iter()
            .map(|ex| {
                let mut dep = Dependent::new_named("version", ex);
                dep.depends_on(&build);
                dep
            })
            .collect::<Vec<_>>();
        let mut tasks = vec![version, clean, install_lint, lint, build, test];
        tasks.extend(exes);
        tasks
    }
}
