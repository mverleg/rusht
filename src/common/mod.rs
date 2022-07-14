use crate::escape::{Charset, HashPolicy, namesafe_line};
use crate::escape::NamesafeArgs;

pub use self::err::fail;
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
mod stdin;
mod read;
mod write;
mod task;
mod re;

pub fn unique_filename(text: &str) -> String {
    namesafe_line(text, &NamesafeArgs {
        charset: Charset::AsciiOnly,
        hash_policy: HashPolicy::Always,
        max_length: 32,
    })
}
