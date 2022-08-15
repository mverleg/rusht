use ::std::env::current_dir;

use ::log::debug;
use ::smallvec::{SmallVec, smallvec};

use crate::common::{LineWriter, Task};
use crate::java::mvnw_args::MvnwArgs;

pub async fn mvnw(mut args: MvnwArgs, writer: &mut impl LineWriter) {
    if args.all_tests {
        debug!("setting --all because of --all-tests");
        args.all = true;
    }
    let args = args;
    // clean
    // all
    // affected_tests
    // all_tests
    // prod_only
    // verbose
    // affected_policy

    let cmd_config = MvnCmdConfig {
        verbose: args.verbose,
        update: args.update,
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
}

impl MvnCmdConfig {
    fn build_cmds(&self) -> SmallVec<[Task; 1]> {
        // clean
        // all
        // affected_tests
        // all_tests
        // prod_only
        // affected_policy
        let mut args = vec![];
        if self.modules {
            for module in self.modules {
                args.push("-pl".to_owned());
                args.push(format!(":{}", module));
            }
            args.push("-am".to_owned())
        }
        if ! self.update {
            args.push("--update-snapshots".to_owned());
        }
        if ! self.verbose {
            args.push("--quiet".to_owned());
        }
        smallvec![Task::new("mvn".to_owned(), args, current_dir().unwrap())]
    }
}
