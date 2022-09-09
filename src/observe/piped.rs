use ::std::future::{Future, join};

use crate::common::{LineReader, StdWriter, Task};
use crate::common::LineWriter;
use crate::ExitStatus;
use crate::observe::chain::{ChainReader, ChainWriter};
use crate::observe::chained;
use crate::observe::piped_args::PipedArgs;

pub async fn piped(args: PipedArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) -> ExitStatus {
    assert!(!args.stderr);
    let (source, sink) = args.cmds.split_once_at(&args.separator);
    let source = source.into_task();
    let sink  = sink.into_task();
    //source.execute_with_stdout();
    let (mut chain_write, mut chain_read) = chained(
        args.pipe_buffer_size.try_into().unwrap_or(usize::MAX));
    let (source_res, sink_res) = join!(
        run_source(source, &mut chain_write, args.stderr),
        //TODO @mverleg: chain_read into this cmd:
        sink.execute_with_stdout_nomonitor(writer, &mut StdWriter::stderr()),
    ).await;
    ExitStatus::max(source_res, sink_res)
}

async fn run_source(task: Task, writer: &mut ChainWriter, is_stderr: bool) -> ExitStatus {
    if is_stderr {
        task.execute_with_stdout_nomonitor(&mut StdWriter::stdout(), writer).await
    } else {
        task.execute_with_stdout_nomonitor(writer, &mut StdWriter::stderr()).await
    }
}
