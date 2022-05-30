#![allow(unused)]  //TODO: TEMPORARY! REMOVE THIS!

pub use cmd_add::add_cmd;
pub use cmd_add::AddArgs;
pub use cmd_add::AddArgsExtra;
pub use cmd_do::do_cmd;
pub use cmd_do::DoArgs;
pub use cmd_drop::drop_cmd;
pub use cmd_drop::DropArgs;
pub use cmd_list::list_cmds;
pub use cmd_list::ListArgs;

mod cmd_add;
mod cmd_do;
mod cmd_list;
mod cmd_drop;
mod cmd_type;
mod cmd_io;
mod tests;
