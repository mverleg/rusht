use crate::common::{LineReader, LineWriter};
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn mon(
    _args: MonArgs,
    _reader: &mut impl LineReader,
    _writer: &mut impl LineWriter,
) -> ExitStatus {
    unimplemented!() //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
