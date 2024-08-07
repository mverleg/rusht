use ::std::collections::HashSet;
use ::std::env;
use ::std::env::current_dir;
use ::std::env::set_current_dir;
use ::std::path::PathBuf;
use ::std::sync::atomic::{AtomicBool, Ordering};

use ::itertools::Itertools;
use ::log::debug;
use ::log::warn;
use ::regex::Regex;
use ::smallvec::SmallVec;

use crate::common::git::git_affected_files_head;
use crate::common::LineWriter;
use crate::common::RegexWatcherWriter;
use crate::common::run_all;
use crate::common::TeeWriter;
use crate::ExitStatus;
use crate::java::MvnCmdConfig;
use crate::java::mvnw_args::AffectedPolicy;
use crate::java::MvnwArgs;

pub async fn mvnw(
    args: MvnwArgs,
    writer: &mut impl LineWriter,
    //TODO @mverleg: ^use LineWriter (has some lifetime issues with run_all)
) -> Result<(), (ExitStatus, String)> {
    assert!(args.threads.unwrap_or(1) >= 1);
    assert!(args.max_memory_mb >= 1);
    if !args.fail_if_added.is_empty() {
        warn!("--fail-if-added not implemented");
    }
    debug!("arguments: {:?}", &args);

    if !args.proj_roots.is_empty() {
        debug!("using multi-dir mode for {} roots", args.proj_roots.len());
        for dir in args.proj_roots.iter() {
            debug!("running for maven root {}", dir.to_string_lossy());
            set_current_dir(dir).map_err(|err| {
                (
                    ExitStatus::err(),
                    format!(
                        "failed to switch working directory to {}, err {}",
                        dir.to_string_lossy(),
                        err
                    ),
                )
            })?;
            mvnw_dir(args.clone(), writer).await?;
        }
        Ok(())
    } else {
        debug!("using current working dir mode (no --proj-root)");
        mvnw_dir(args, writer).await
    }
}

async fn mvnw_dir(
    args: MvnwArgs,
    writer: &mut impl LineWriter,
) -> Result<(), (ExitStatus, String)> {
    let cwd = current_dir().expect("could not determine working directory");
    if !PathBuf::from("pom.xml").is_file() {
        return Err((
            ExitStatus::err(),
            "must be run from a maven project directory (containing pom.xml)".to_owned(),
        ));
    }
    //TODO @mverleg: affected_policy

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
    let rebuild_if_match = args
        .rebuild_if_match
        .iter()
        .cloned()
        .collect::<SmallVec<[Regex; 1]>>();
    //let is_offline = !args.update;
    let cmd_config = build_config(cwd, java_home, args).map_err(|err| (ExitStatus::err(), err))?;

    debug!("command config: {:?}", cmd_config);
    let cmds = cmd_config.build_cmds();
    if show_cmds_only {
        for cmd in cmds {
            if let Some(task) = cmd.task() {
                writer.write_line(task.as_str()).await;
            }
        }
        return Ok(());
    }
    let has_pattern = AtomicBool::new(false);
    let mut watcher = RegexWatcherWriter::new(rebuild_if_match, |_| {
        has_pattern.store(true, Ordering::Release)
    });
    let mut tee_writer = TeeWriter::new(writer, &mut watcher);
    let mut status = run_all(cmds, &mut tee_writer).await;
    if has_pattern.load(Ordering::Acquire) {
        eprintln!("going to clean rebuild because a --rebuild-if-match was matched");
        let mut clean_config = cmd_config;
        clean_config.clean = true;
        //TODO @mverleg: debug^^^
        //TODO @mverleg: this hangs if clean is true, but doesn't if false (it just fails again)
        //TODO @mverleg: it reports 'no input on stdin so far' but is probably from grab
        //TODO @mverleg: once this problem has triggered, it will hang on clean even if clean is the first command (i.e. no rebuild needed)
        //TODO @mverleg: this seems to always happen on selective clean, i.e. with profile when clean runs as a separate mvn command - not just for re-run
        //TODO @mverleg: it also happens when both checkstyle and -v (version) are enabled, which also triggers two simultaneous mvn commands - just version (-vL) does not
        //dbg!(&io::stdout()); //TODO @mverleg: TEMPORARY! REMOVE THIS!
        status = run_all(clean_config.build_cmds(), writer).await;
    }
    if status.is_ok() {
        Ok(())
    } else {
        Err((status, "".to_owned()))
    }

    //TODO @mverleg: special warning if fails because of offline mode

    //    if !status.success() {
    //         if let Some(task) = cmd.task() {
    //             if is_offline && task.cmd == "mvn" {
    //                 eprintln!("note: failed in offline mode, use -U for online")
    //             }
    //         }
    //         return Err((status.code(), "".to_owned()));
    //     }
    // for (nr, cmd) in cmds.iter().enumerate() {
    //     // writer
    //     //     .write_line(format!(
    //     //         "going to run [{}/{}]: {}",
    //     //         nr + 1,
    //     //         cmds.len(),
    //     //         cmd.as_str()
    //     //     ))
    //     //     .await;
    //     if show_cmds_only {
    //         continue;
    //     }
    //     let status = cmd.execute(false);
    //     if !status.success() {
    //         if let Some(task) = cmd.task() {
    //             if is_offline && task.cmd == "mvn" {
    //                 eprintln!("note: failed in offline mode, use -U for online")
    //             }
    //         }
    //         return Err((status.code(), "".to_owned()));
    //     }
    // }
}

fn build_config(cwd: PathBuf, java_home: PathBuf, args: MvnwArgs) -> Result<MvnCmdConfig, String> {
    let modules = if !args.modules.is_empty() {
        assert!(!args.all, "do not use --all if modules are explicitly specified");
        Some(args.modules.iter()
            .flat_map(|m| m.split(','))
            .map(|m| m.trim().to_owned())
            .sorted()
            .unique()
            .collect())
    } else if args.all {
        None
    } else {
        warn!("using --all as default; this behaviour might change");
        None
    };

    if args.affected_policy == AffectedPolicy::None && args.modules.is_empty() {
        return Err("--affected=none only makes sense with --module".to_owned())
    } else if args.affected_policy != AffectedPolicy::Head {
        eprintln!("ignoring provided --affected and using --affected=head instead");
        //TODO @mverleg: ^
    }
    let (changed_files, _) = git_affected_files_head(&cwd).unwrap_or_else(|err| {
        eprintln!("skipping git integration: {}", err);
        (HashSet::new(), HashSet::new())
    });
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

    // phase is a vec to allow overriding, but really we only care about the last one
    let phase_override= if let Some(phase) = &args.phase.last() {
        assert!(!args.install, "--phase and --install should not be combined");
        Some((*phase).to_owned())
    } else if args.install {
        Some("install".to_owned())
    } else {
        None
    };

    let cmd_config = MvnCmdConfig {
        changed_files,
        modules,
        no_build_deps: args.no_build_deps,
        tests: args.test(),
        lint: !args.no_lint,
        checkstyle_version: "10.6.0".to_string(),
        verbose: args.verbose,
        update: args.update,
        clean: args.clean,
        phase_override,
        execs: args.execs,
        profiles: args.profiles.into_iter().sorted().unique().collect(),
        threads: args.threads.unwrap_or_else(|| num_cpus::get() as u32),
        max_memory_mb: args.max_memory_mb,
        max_exec_memory_mb: args.max_exec_memory_mb.unwrap_or(args.max_memory_mb),
        mvn_exe: args.mvn_exe,
        mvn_arg: args.mvn_args.into_iter().sorted().collect(),
        mvn_defs: args.mvn_defs.into_iter().sorted().collect(),
        java_home,
        cwd,
    };
    Ok(cmd_config)
}
