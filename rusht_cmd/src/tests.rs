use ::rand::Rng;
use ::rand::thread_rng;

use crate::{add_cmd, AddArgs, AddArgsExtra, drop_cmd, DropArgs};

fn init_test() -> String {
    env_logger::init();
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
    add_cmd(AddArgs {
        namespace: namespace.to_string(),
        quiet: false,
        end: false,
        skip_validation: false,
        lines: false,
        lines_with: Some("%".to_owned()),
        cmd: AddArgsExtra::Cmd(vec!["echo".to_owned(), "hello".to_owned(), "%".to_owned()])
    });
    drop_cmd(DropArgs {
        namespace,
        all: true,
        count: 0,
        end: false,
        quiet: false,
    });
}
