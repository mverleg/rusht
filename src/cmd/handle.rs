use crate::common::stdin_lines;
use crate::common::EmptyLineHandling;
use crate::ExitStatus;

use super::list_cmds;
use super::ListArgs;
use super::ListErr;
use super::{add_cmd, AddArgs};
use super::{do_cmd, DoArgs};
use super::{drop_cmd, DropArgs};

pub fn handle_add(mut args: AddArgs) -> ExitStatus {
    if args.lines {
        args.lines_with = Some("{}".to_owned());
    }
    add_cmd(args, || stdin_lines(EmptyLineHandling::Drop));
    ExitStatus::ok()
}

pub fn handle_do(args: DoArgs) -> ExitStatus {
    assert!(!args.parallel > 1, "parallel not implemented"); // TODO

    let all_ok = do_cmd(args);
    ExitStatus::of_is_ok(all_ok)
}

pub fn handle_drop(args: DropArgs) -> ExitStatus {
    assert!(!args.end, "end not implemented"); //TODO
    drop_cmd(args);
    ExitStatus::ok()
}

pub fn handle_list(args: ListArgs) -> ExitStatus {
    match list_cmds(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
            ExitStatus::ok()
        }
        Err(ListErr::Empty) => ExitStatus::err(),
    }
}
