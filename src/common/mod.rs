use crate::escape::NamesafeArgs;
use crate::escape::{namesafe_line, Charset, HashPolicy};

pub use self::err::fail;
pub use self::err::ExitStatus;
pub use self::git::git_affected_files_head;
pub use self::git::git_head_ref;
pub use self::re::get_first_match_or_all;
pub use self::re::get_matches;
pub use self::read::LineReader;
pub use self::read::StdinReader;
pub use self::read::VecReader;
pub use self::stdin::stdin_lines;
pub use self::stdin::EmptyLineHandling;
pub use self::task::CommandArgs;
pub use self::task::Task;
pub use self::write::CollectorWriter;
pub use self::write::FirstItemWriter;
pub use self::write::LineWriter;
pub use self::write::StdoutWriter;
pub use self::write::VecWriter;
pub use self::write::TeeWriter;

mod err;
mod git;
mod re;
mod read;
mod stdin;
mod task;
mod write;

pub fn unique_filename(text: &str) -> String {
    namesafe_line(
        text,
        &NamesafeArgs {
            charset: Charset::AsciiOnly,
            hash_policy: HashPolicy::Always,
            max_length: 32,
            ..Default::default()
        },
    )
}
