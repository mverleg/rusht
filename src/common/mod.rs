pub use self::err::fail;
pub use self::files::unique_filename;
pub use self::re::get_matches;
pub use self::stdin::EmptyLineHandling;
pub use self::stdin::stdin_lines;
pub use self::task::CommandArgs;
pub use self::task::Task;

mod err;
mod files;
mod stdin;
mod task;
mod re;
