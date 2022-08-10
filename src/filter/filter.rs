use ::log::debug;

use crate::common::{get_first_match_or_all, LineReader, LineWriter};
use crate::filter::FilterArgs;

//TODO @mverleg: pattern {} in cmd!
//TODO @mverleg: parallel?
pub async fn filter(args: FilterArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) {
    let expect_success = !args.invert;
    let base_task = args.cmd.clone().into_task();
    while let Some(line) = reader.read_line().await {
        let arg = get_first_match_or_all(&args.by, line);
        let mut task = base_task.clone();
        task.push_arg(arg);
        let status = task.execute(true);
        if expect_success == status.success() {
            debug!("keep line {} after task {} (code: {})", line,
                task.as_cmd_str(), status.code().unwrap_or(-1));
            writer.write_line(line).await;
        } else {
            debug!("discard line {} after task {} (code: {})", line,
                task.as_cmd_str(), status.code().unwrap_or(-1));
        }
    }
}

//TODO @mverleg: tests
