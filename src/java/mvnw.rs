use ::std::env;
use ::std::env::current_dir;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::log::debug;

use crate::common::{git_affected_files_head, LineWriter};
use crate::java::mvnw_args::AffectedPolicy;
use crate::java::MvnCmdConfig;
use crate::java::MvnwArgs;
use crate::ExitStatus;

pub async fn mvnw(
    args: MvnwArgs,
    writer: &mut impl LineWriter,
) -> Result<(), (ExitStatus, String)> {
    assert!(args.threads.unwrap_or(1) >= 1);
    assert!(args.max_memory_mb >= 1);
    debug!("arguments: {:?}", &args);
    if !args.all {
        return Err((ExitStatus::err(), "--all required for now".to_owned())); //TODO @mverleg: --all required for now
    }
    let cwd = current_dir().expect("could not determine working directory");
    if !PathBuf::from("pom.xml").is_file() {
        return Err((
            ExitStatus::err(),
            "must be run from a maven project directory (containing pom.xml)".to_owned(),
        ));
    }
    let args = args;
    // //TODO @mverleg: affected_policy

    debug!("JAVA_HOME = {:?}", env::var("JAVA_HOME"));
    let java_home = PathBuf::try_from(env::var("JAVA_HOME").map_err(|err| {
        (
            ExitStatus::err(),
            format!("could not read JAVA_HOME from env, err: {}", err),
        )
    })?)
    .map_err(|err| {
        (
            ExitStatus::err(),
            format!("JAVA_HOME env does not contain a valid path, err: {}", err),
        )
    })?;
    if !java_home.is_dir() {
        return Err((
            ExitStatus::err(),
            format!(
                "JAVA_HOME directory does not exist at {}",
                java_home.to_string_lossy()
            ),
        ));
    }

    let show_cmds_only = args.show_cmds_only;
    let is_offline = !args.update;
    let cmd_config = builds_cmds(cwd, java_home, args).map_err(|err| (ExitStatus::err(), err))?;

    debug!("command config: {:?}", cmd_config);
    let cmds = cmd_config.build_cmds();
    for (nr, cmd) in cmds.iter().enumerate() {
        writer
            .write_line(format!(
                "going to run [{}/{}]: {}",
                nr + 1,
                cmds.len(),
                cmd.as_str()
            ))
            .await;
        if show_cmds_only {
            continue;
        }
        let status = cmd.execute(false);
        if !status.success() {
            if is_offline && cmd.cmd == "mvn" {
                eprintln!("note: failed in offline mode, use -U for online")
            }
            return Err((ExitStatus::of_err(status.code()), "".to_owned()));
        }
    }

    Ok(())
}

fn builds_cmds(cwd: PathBuf, java_home: PathBuf, args: MvnwArgs) -> Result<MvnCmdConfig, String> {
    let modules = if args.all {
        vec![]
    } else {
        vec!["omm-goat".to_owned()]; //TODO @mverleg: TEMPORARY! REMOVE THIS!
        unimplemented!()
    };

    if args.affected_policy != AffectedPolicy::Head {
        eprintln!("ignoring provided --affected and using --affected=head instead");
        //TODO @mverleg: ^
    }
    let (changed_files, _) = git_affected_files_head(&cwd)?;
    if let Some(example) = changed_files.iter().next() {
        debug!(
            "found {} affected files for {}, e.g. {}",
            changed_files.len(),
            args.affected_policy,
            example.to_string_lossy()
        );
    } else {
        debug!(
            "no affected files for {}, e.g. {}",
            changed_files.len(),
            args.affected_policy
        );
    }

    let cmd_config = MvnCmdConfig {
        changed_files,
        modules,
        tests: args.test(),
        lint: !args.no_lint,
        checkstyle_version: "8.1".to_string(),
        verbose: args.verbose,
        update: args.update,
        clean: args.clean,
        install: args.install,
        profiles: args.profiles.into_iter().sorted().unique().collect(),
        threads: args.threads.unwrap_or_else(|| num_cpus::get() as u32),
        max_memory_mb: args.max_memory_mb,
        mvn_exe: args.mvn_exe,
        mvn_arg: args.mvn_args.into_iter().sorted().collect(),
        java_home,
        cwd,
    };
    Ok(cmd_config)
}
