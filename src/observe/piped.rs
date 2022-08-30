use crate::common::LineReader;
use crate::common::LineWriter;
use crate::ExitStatus;
use crate::observe::piped_args::PipedArgs;

pub async fn piped(args: PipedArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) -> ExitStatus {
    unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
