pub use self::err::fail;
pub use self::stdin::stdin_lines;
pub use self::stdin::EmptyLineHandling;
pub use self::task::CommandArgs;
pub use self::task::Task;
pub use self::files::to_filename;

mod err;
mod stdin;
mod task;
mod files;
