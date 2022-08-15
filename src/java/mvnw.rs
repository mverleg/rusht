use ::std::env::current_dir;
use std::path::PathBuf;

use ::log::debug;
use ::smallvec::{SmallVec, smallvec};

use crate::common::{LineWriter, Task};
use crate::java::mvnw_args::MvnwArgs;

pub async fn mvnw(mut args: MvnwArgs, writer: &mut impl LineWriter) {
    assert!(!(args.prod_only && args.tests));
    assert!(args.threads.unwrap_or(1) >= 1);
    assert!(args.max_memory_mb >= 1);
    if ! args.all {
        unimplemented!("--all is required for now")  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    }
    if args.tests {
        debug!("setting --all because of --all-tests");
        args.all = true;
    }
    debug!("arguments: {:?}", &args);
    let args = args;
    // affected_tests
    // all_tests
    // prod_only
    // affected_policy

    // threads
    // max_memory
    // mvn_exe
    // mvn_arg

    let modules = if args.all {
        vec![]
    } else {
        unimplemented!()
    };
    let cmd_config = MvnCmdConfig {
        modules,
        verbose: args.verbose,
        update: args.update,
        clean: args.clean,
        install: args.install,
        prod_only: args.prod_only,
        threads: args.threads.unwrap_or_else(|| num_cpus::get() as u32),
        max_memory_mb: args.max_memory_mb,
        mvn_exe: args.mvn_exe,
        mvn_arg: args.mvn_arg,
        cwd: current_dir().unwrap(),
    };
    for cmd in cmd_config.build_cmds() {
        if args.show_cmds_only {
            writer.write_line(cmd.as_cmd_str());
        } else {
            let status = cmd.execute(!args.verbose);
            unimplemented!()  //TODO @mverleg:
        }
    }

    unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}

struct MvnCmdConfig {
    /// Which modules to build. Empty means everything.
    modules: Vec<String>,
    verbose: bool,
    update: bool,
    clean: bool,
    install: bool,
    prod_only: bool,
    threads: u32,
    max_memory_mb: u32,
    mvn_exe: String,
    mvn_arg: Vec<String>,
    cwd: PathBuf,
}

impl MvnCmdConfig {
    fn build_cmds(&self) -> SmallVec<[Task; 1]> {
        let do_tests = false;  //TODO @mverle
        // tests
        // max_memory

        let mut cmds = smallvec![];
        let mut args = vec![];
        if self.verbose {
            cmds.push(self.make_task(vec!["--version".to_owned()]));
        }

        // Clean
        if self.clean && self.modules.is_empty() {
            args.push("clean".to_owned());
        } else {
            let mut clean_args = vec!["clean".to_owned()];
            if ! self.verbose {
                clean_args.push("--quiet".to_owned());
            }
            cmds.push(self.make_task(clean_args));
        }

        // Determine maven stage
        let stage = if self.install {
            "install"
        } else if do_tests {
            "test"
        } else {
            "compile"
        };
        args.push(stage.to_owned());

        // Affected build modules
        if ! self.modules.is_empty() {
            for module in &self.modules {
                args.push("-pl".to_owned());
                args.push(format!(":{}", module));
            }
            args.push("-am".to_owned())
        }

        // Modifier flags
        args.push(format!("--threads={}", self.threads));
        if ! self.update {
            args.push("--update-snapshots".to_owned());
        }
        if ! self.verbose {
            args.push("--quiet".to_owned());
        }
        if self.prod_only {
            args.push("-Dmaven.test.skip=true".to_owned());
        }
        if do_tests {
            args.push("-Dparallel=all".to_owned());
            args.push("-DperCoreThreadCount=false".to_owned());
            args.push(format!("-DthreadCount={}", if self.threads > 1 { 4 * self.threads } else { 1 }));
        }

        // Default optimization flags
        args.push("-DskipITs".to_owned());
        args.push("-Dmanagedversions.skip=true".to_owned());
        args.push("-Dmanagedversions.failOnError=false".to_owned());
        args.push("-Denforcer.skip=true".to_owned());
        args.push("-Ddatabase.skip=true".to_owned());
        args.push("-Dsurefire.printSummary=false".to_owned());
        args.push("-DfailIfNoTests=false".to_owned());
        args.push("-Dmaven.javadoc.skip=true".to_owned());

        cmds.push(self.make_task(args));
        cmds
    }

    fn make_task(&self, mut args: Vec<String>) -> Task {
        args.extend_from_slice(&self.mvn_arg);
        Task::new(self.mvn_exe.to_owned(), args, self.cwd.to_owned())
    }
}
