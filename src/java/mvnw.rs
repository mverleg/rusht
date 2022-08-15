use ::std::env::current_dir;
use ::std::path::PathBuf;

use ::log::debug;

use crate::common::LineWriter;
use crate::java::MvnCmdConfig;
use crate::java::MvnwArgs;

pub async fn mvnw(args: MvnwArgs, writer: &mut impl LineWriter) -> Result<(), String> {
    assert!(!(args.prod_only && args.tests));
    assert!(args.threads.unwrap_or(1) >= 1);
    assert!(args.max_memory_mb >= 1);
    debug!("arguments: {:?}", &args);
    if ! PathBuf::from("pom.xml").is_file() {
        return Err("must be run from a maven project directory (containing pom.xml)".to_owned())

    }
    let args = args;
    // //TODO @mverleg: affected_policy

    let modules = if args.all {
        vec![]
    } else {
        vec!["omm-goat".to_owned()]  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    };
    let cmd_config = MvnCmdConfig {
        modules,
        tests: args.tests,
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
            writer.write_line(cmd.as_cmd_str()).await;
        } else {
            debug!("{}", cmd.as_cmd_str());
            //TODO @mverleg: maybe some logging
            let status = cmd.execute(false);
            if ! status.success() {
                return Err(format!("command {} failed with code {}",
                        cmd.as_cmd_str(), status.code().unwrap_or(-1)))
            }
        }
    }

    Ok(())
}
