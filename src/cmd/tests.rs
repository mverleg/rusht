use ::std::io::stdin;
use ::std::sync::Once;
use std::fs;
use std::io::Read;

use ::rand::thread_rng;
use ::rand::Rng;

use ::tempfile::tempfile;
use tempfile::NamedTempFile;

use crate::cmd::{
    add_cmd, do_cmd, drop_cmd, list_cmds, AddArgs, AddArgsExtra, DoArgs, DropArgs, ListArgs,
};

static INIT: Once = Once::new();

fn init_test() -> String {
    INIT.call_once(|| {
        env_logger::init();
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
            skip_validation: false,
            lines: false,
            lines_with: Some("%".to_owned()),
            cmd: AddArgsExtra::Cmd(vec!["echo".to_owned(), "hello".to_owned(), "%".to_owned()]),
        },
        || vec!["Leonardo".to_owned(), "Benjamin".to_owned()],
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
            "echo hello Benjamin  # 1".to_owned(),
            "echo hello Leonardo  # 2".to_owned(),
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
            skip_validation: false,
            lines: false,
            lines_with: None,
            cmd: AddArgsExtra::Cmd(args),
        },
        std::vec::Vec::new,
    );
}

#[test]
fn onebyone_add_run() {
    let namespace = init_test();
    let mut outfile = NamedTempFile::new();
    let out_path = outfile
        .as_ref()
        .unwrap()
        .path()
        .to_string_lossy()
        .to_string();
    add_one(&namespace, vec!["cat".to_owned(), out_path.clone()]);
    add_one(
        &namespace,
        vec![
            "echo".to_owned(),
            "bye".to_owned(),
            "world".to_owned(),
            ">>".to_owned(),
            out_path.clone(),
        ],
    );
    add_one(
        &namespace,
        vec![
            "echo".to_owned(),
            "hello".to_owned(),
            "world".to_owned(),
            ">>".to_owned(),
            out_path.clone(),
        ],
    );
    let out = list_cmds(ListArgs {
        namespace: namespace.to_owned(),
        file_path: false,
        count: None,
        exit_code: false,
    })
    .unwrap();
    assert_eq!(out.len(), 3);
    assert!(out[0].starts_with("echo hello world >> "));
    assert!(out[1].starts_with("echo bye world >> "));
    do_cmd(DoArgs {
        namespace: namespace.to_owned(),
        count: 1,
        autorun: true,
        parallel: false,
        always_pop: false,
        keep: false,
        quiet: false,
    });
    let out = list_cmds(ListArgs {
        namespace,
        file_path: false,
        count: None,
        exit_code: true,
    });
    assert!(out.is_err());
    let outfile_content = fs::read_to_string(out_path).unwrap();
    assert_eq!(outfile_content, "abc");
    drop(outfile.unwrap());
}
