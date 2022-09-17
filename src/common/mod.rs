use crate::escape::NamesafeArgs;
use crate::escape::{namesafe_line, Charset, HashPolicy};

pub use self::cmd_args::CommandArgs;
pub use self::dependent::run_all;
pub use self::dependent::Dependent;
pub use self::err::fail;
pub use self::err::ExitStatus;
pub use self::git::git_affected_files_head;
pub use self::git::git_head_ref;
pub use self::re::get_first_match_or_all;
pub use self::re::get_matches;
pub use self::read::LineReader;
pub use self::read::RejectStdin;
pub use self::read::StdinReader;
pub use self::read::VecReader;
pub use self::stdin::stdin_lines;
pub use self::stdin::EmptyLineHandling;
pub use self::task::Task;
pub use self::which::resolve_executable;
pub use self::write::CollectorWriter;
pub use self::write::FirstItemWriter;
pub use self::write::LineWriter;
pub use self::write::RegexWatcherWriter;
pub use self::write::StdWriter;
pub use self::write::TeeWriter;
pub use self::write::VecWriter;

mod async_gate;
mod cmd_args;
mod dependent;
mod err;
mod exec;
mod git;
mod re;
mod read;
mod stdin;
mod task;
mod which;
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
