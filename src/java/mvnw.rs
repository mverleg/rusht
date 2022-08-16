use ::std::env;
use ::std::env::current_dir;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::log::debug;

use crate::common::LineWriter;
use crate::java::MvnCmdConfig;
use crate::java::mvnw_args::TestMode;
use crate::java::MvnwArgs;

pub async fn mvnw(args: MvnwArgs, writer: &mut impl LineWriter) -> Result<(), String> {
    let mut test_mode = args.test();
    if args.prod_only {
        if !args.is_test_arg_provided() {
            debug!("disabling tests (--no-tests) because --prod-only was requested");
            test_mode = TestMode::None;
        } else {
            if test_mode != TestMode::None {
                return Err("cannot run tests in --prod-only mode".to_owned());
            }
        }
    }
    assert!(args.threads.unwrap_or(1) >= 1);
    assert!(args.max_memory_mb >= 1);
    debug!("arguments: {:?}", &args);
    if ! args.all {
        return Err("--all required for now".to_owned());  //TODO @mverleg: --all required for now
    }
    if !PathBuf::from("pom.xml").is_file() {
        return Err("must be run from a maven project directory (containing pom.xml)".to_owned());
    }
    let args = args;
    // //TODO @mverleg: affected_policy

    let modules = if args.all {
        vec![]
    } else {
        vec!["omm-goat".to_owned()]; //TODO @mverleg: TEMPORARY! REMOVE THIS!
        unimplemented!()
    };
    debug!("JAVA_HOME = {:?}", env::var("JAVA_HOME"));
    let java_home = PathBuf::try_from(env::var("JAVA_HOME")
        .map_err(|err| format!("could not read JAVA_HOME from env, err: {}", err))?)
        .map_err(|err| format!("JAVA_HOME env does not contain a valid path, err: {}", err))?;
    if ! java_home.is_dir() {
        return Err(format!("JAVA_HOME directory does not exist at {}", java_home.to_string_lossy()));
    }
    let java_home = java_home.to_str().ok_or_else(|| "JAVA_HOME path is not unicode".to_owned())?.to_owned();
    let cmd_config = MvnCmdConfig {
        modules,
        tests,
        verbose: args.verbose,
        update: args.update,
        clean: args.clean,
        install: args.install,
        prod_only: args.prod_only,
        profiles: args.profiles.into_iter().sorted().unique().collect(),
        threads: args.threads.unwrap_or_else(|| num_cpus::get() as u32),
        max_memory_mb: args.max_memory_mb,
        mvn_exe: args.mvn_exe,
        mvn_arg: args.mvn_args.into_iter().sorted().collect(),
        java_home,
        cwd: current_dir().unwrap(),
    };

    debug!("command config: {:?}", cmd_config);
    let cmds = cmd_config.build_cmds();
    for (nr, cmd) in cmds.iter().enumerate() {
        writer.write_line(format!("going to run [{}/{}]: {}", nr + 1, cmds.len(), cmd.as_str())).await;
        if args.show_cmds_only {
            continue;
        }
        let status = cmd.execute(false);
        if !status.success() {
            if ! args.update {
                eprintln!("note: failed in offline mode, use -U for online")
            }
            return Err(format!(
                "command {} failed with code {}",
                cmd.as_str(),
                status.code().unwrap_or(-1)
            ));
        }
    }

    Ok(())
}
