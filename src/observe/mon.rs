use crate::common::{LineReader, LineWriter};
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn mon(
    args: MonArgs,
    _reader: &mut impl LineReader,
    _writer: &mut impl LineWriter,
) -> ExitStatus {
    let task = args.cmd.into_task();
    let status = task.execute(false);
    ExitStatus::of_err(status.code())
    //TODO @mverleg:
}
