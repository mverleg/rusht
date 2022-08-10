use ::std::collections::HashSet;
use ::std::env::current_dir;
use ::std::io::Read;
use ::std::io::stdin;
use ::std::path::PathBuf;
use ::std::thread::spawn;

use ::clap::StructOpt;
use ::log::debug;

use crate::common::{CommandArgs, fail, get_first_match_or_all, LineReader, LineWriter, Task};
use crate::filter::FilterArgs;

//TODO @mverleg: parallel?
pub async fn filter(args: FilterArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) {
    while let Some(line) = reader.read_line().await {
        let arg = get_first_match_or_all(&args.by, line);
        let task = args.cmd.into_task();
        let status = task.execute(false);
        if args.invert != status.success() {
            writer.write_line(line);
        }
    }
}
