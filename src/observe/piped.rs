use crate::common::LineReader;
use crate::common::LineWriter;
use crate::ExitStatus;
use crate::observe::piped_args::PipedArgs;

pub async fn piped(args: PipedArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) -> ExitStatus {
    let (source, sink) = args.cmds.split_once_at(&args.separator);


    unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
