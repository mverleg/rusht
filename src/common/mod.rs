pub use self::err::fail;
pub use self::files::unique_filename;
pub use self::re::get_matches;
pub use self::read::LineReader;
pub use self::read::StdinReader;
pub use self::read::VecReader;
pub use self::stdin::EmptyLineHandling;
pub use self::stdin::stdin_lines;
pub use self::task::CommandArgs;
pub use self::task::Task;
pub use self::write::FirstItemWriter;
pub use self::write::LineWriter;
pub use self::write::StdoutWriter;
pub use self::write::VecWriter;

mod err;
mod files;
//#[deprecated]   //TODO @mark:
mod stdin;
mod read;
mod write;
mod task;
mod re;
mod async;
