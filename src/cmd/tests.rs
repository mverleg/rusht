use ::std::fs;
use ::std::sync::Once;

use ::rand::thread_rng;
use ::rand::Rng;
use ::tempfile::NamedTempFile;

use crate::cmd::{add_cmd, do_cmd, drop_cmd, list_cmds, AddArgs, DoArgs, DropArgs, ListArgs};
use crate::common::CommandArgs;

static INIT: Once = Once::new();

fn init_test() -> String {
    INIT.call_once(|| {
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
        );
    });
    let mut rng = thread_rng();
    let namespace = format!("unit_test_{}", rng.gen::<u32>());
    drop_cmd(DropArgs {
        namespace: namespace.to_string(),
        all: true,
        count: 1,
        end: false,
        quiet: true,
    });
    namespace
}

#[test]
fn batch_add_drop() {
    let namespace = init_test();
    add_cmd(
        AddArgs {
            namespace: namespace.to_string(),
            quiet: false,
            end: false,
            lines: false,
            lines_with: Some("%".to_owned()),
            unique: true,
            working_dir: None,
            cmd: CommandArgs::Cmd(vec!["print".to_owned(), "hello".to_owned(), "%".to_owned()]),
        },
        || {
            vec![
                "Leonardo".to_owned(),
                "Benjamin".to_owned(),
                "Leonardo".to_owned(),
            ]
        },
    );
    let out = list_cmds(ListArgs {
        namespace: namespace.to_owned(),
        file_path: false,
        count: None,
        exit_code: false,
    })
    .unwrap();
    assert_eq!(
        out,
        vec![
            "/usr/bin/print hello Benjamin  # 1".to_owned(),
            "/usr/bin/print hello Leonardo  # 2".to_owned(),
        ]
    );
    drop_cmd(DropArgs {
        namespace: namespace.to_owned(),
        all: true,
        count: 0,
        end: false,
        quiet: false,
    });
    let out = list_cmds(ListArgs {
        namespace,
        file_path: false,
        count: None,
        exit_code: true,
    });
    assert!(out.is_err());
}

fn add_one(namespace: &str, args: Vec<String>) {
    add_cmd(
        AddArgs {
            namespace: namespace.to_string(),
            quiet: false,
            end: false,
            lines: false,
            lines_with: None,
            unique: false,
            working_dir: None,
            cmd: CommandArgs::Cmd(args),
        },
        std::vec::Vec::new,
    );
}

#[test]
fn onebyone_add_run() {
    let namespace = init_test();
    let outfile = NamedTempFile::new();
    let out_path = outfile
        .as_ref()
        .unwrap()
        .path()
        .to_string_lossy()
        .to_string();
    add_one(
        &namespace,
        vec![
            "sh".to_owned(),
            "-c".to_owned(),
            format!("echo hello world >> {}", &out_path),
        ],
    );
    add_one(
        &namespace,
        vec![
            "sh".to_owned(),
            "-c".to_owned(),
            format!("echo bye world >> {}", &out_path),
        ],
    );
    add_one(&namespace, vec!["cat".to_owned(), out_path.clone()]);
    let out = list_cmds(ListArgs {
        namespace: namespace.to_owned(),
        file_path: false,
        count: None,
        exit_code: false,
    })
    .unwrap();
    assert_eq!(out.len(), 3);
    assert!(out[1].contains("echo bye world >> "));
    assert!(out[2].contains("echo hello world >> "));
    do_cmd(DoArgs {
        namespace: namespace.to_owned(),
        count: 1,
        all: true,
        parallel: 1,
        restart_running: false,
        continue_on_error: false,
        drop_failed: false,
        keep_successful: false,
        quiet: false,
        allow_empty: false,
    });
    let out = list_cmds(ListArgs {
        namespace,
        file_path: false,
        count: None,
        exit_code: true,
    });
    assert!(out.is_err());
    let outfile_content = fs::read_to_string(out_path).unwrap();
    assert_eq!(outfile_content, "hello world\nbye world\n");
    drop(outfile.unwrap());
}
