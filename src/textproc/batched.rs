use crate::common::{LineReader, LineWriter};
use crate::ExitStatus;
use crate::textproc::batched_args::BatchedArgs;

pub async fn batched(
    args: BatchedArgs,
    _reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    todo!();
}
