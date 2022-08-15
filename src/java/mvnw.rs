use ::std::env::current_dir;
use std::path::Path;

use ::log::debug;
use ::smallvec::{SmallVec, smallvec};

use crate::common::{LineWriter, Task};
use crate::java::mvnw_args::MvnwArgs;

pub async fn mvnw(mut args: MvnwArgs, writer: &mut impl LineWriter) {
    assert!(!(args.prod_only && args.tests));
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
        prod_only: args.prod_only,
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
}

impl MvnCmdConfig {
    fn build_cmds(&self) -> SmallVec<[Task; 1]> {
        let cwd = current_dir().unwrap();
        let do_tests = false;  //TODO @mverle
        // affected_tests
        // all_tests

        let mut cmds = smallvec![];
        let mut args = vec![];

        // Clean
        if self.clean && self.modules.is_empty() {
            args.push("clean".to_owned());
        } else {
            let mut clean_args = vec!["clean".to_owned()];
            if ! self.verbose {
                clean_args.push("--quiet".to_owned());
            }
            cmds.push(make_task(clean_args, &cwd));
        }

        // Determine maven stage
        let stage = if self.install {
            "install"
        } else if do_tests {
            "test"
        } else {
            "compile"
        };
        args.push(stage);

        // Affected build modules
        if ! self.modules.is_empty() {
            for module in &self.modules {
                args.push("-pl".to_owned());
                args.push(format!(":{}", module));
            }
            args.push("-am".to_owned())
        }

        // Modifier flags
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
            args.push("-DthreadCount=30".to_owned());
        }

        // Default optimizer flags


        cmds.push(make_task(args, &cwd));
        cmds
    }
}

fn make_task(args: Vec<String>, cwd: &Path) -> Task {
    Task::new("mvn".to_owned(), args, cwd.to_owned())
}
