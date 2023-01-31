use ::log::debug;

use crate::common::{LineReader, LineWriter};
use crate::ExitStatus;
use crate::textproc::batched_args::BatchedArgs;

pub async fn batched(
    args: BatchedArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    let batch_size: usize = args.batch_size.try_into().expect("usize too small");
    let mut batch = Vec::with_capacity(batch_size);
    let mut batch_nr = 0;
    while let Some(line) = reader.read_line().await {
        if batch.len() >= batch_size {
            debug!("handling batch #{} of size {}", batch_nr, batch.len());
            batch_nr += 1;
            todo!()
        }
        batch.push(line);
        debug_assert!(batch.len() <= batch_size);
    }
    todo!()
}
