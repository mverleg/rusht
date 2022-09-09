use crate::common::LineReader;
use crate::common::LineWriter;
use crate::ExitStatus;
use crate::observe::piped_args::PipedArgs;

pub async fn piped(args: PipedArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) -> ExitStatus {
    assert!(!args.stderr);
    let (source, sink) = args.cmds.split_once_at(&args.separator);
    let source = source.into_task();
    let sink  = sink.into_task();
    //source.execute_with_stdout();

    unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
