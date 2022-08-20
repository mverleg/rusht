use ::std::process::exit;

use crate::common::EmptyLineHandling;
use crate::common::stdin_lines;

use super::{add_cmd, AddArgs};
use super::{do_cmd, DoArgs};
use super::{drop_cmd, DropArgs};
use super::list_cmds;
use super::ListArgs;
use super::ListErr;

pub fn handle_add(mut args: AddArgs) -> ExitCode {
    if args.lines {
        args.lines_with = Some("{}".to_owned());
    }
    add_cmd(args, || stdin_lines(EmptyLineHandling::Drop));
}

pub fn handle_do(args: DoArgs) {
    assert!(!args.parallel > 1, "parallel not implemented"); // TODO

    let all_ok = do_cmd(args);
    exit(if all_ok { 0 } else { 1 })
}

pub fn handle_drop(args: DropArgs) {
    assert!(!args.end, "end not implemented"); //TODO
    drop_cmd(args);
}

pub fn handle_list(args: ListArgs) {
    match list_cmds(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
        }
        Err(ListErr::Empty) => {
            exit(1);
        }
    }
}
