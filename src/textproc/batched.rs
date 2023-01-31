use ::log::debug;

use crate::common::{LineReader, LineWriter, StdWriter, Task};
use crate::textproc::batched_args::BatchedArgs;

pub async fn batched(
    args: BatchedArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> Result<(), String> {
    let batch_size: usize = args.batch_size.try_into().expect("usize too small");
    let mut batch = Vec::with_capacity(batch_size);
    let task = args.cmd.into_task();
    let mut batch_nr = 0;
    while let Some(line) = reader.read_line().await {
        if batch.len() >= batch_size {
            debug!("handling batch #{} of size {}", batch_nr, batch.len());
            batch_nr += 1;
            run_batch(&batch, &task, writer).await?;
        }
        batch.push(line.to_owned());
        // ^ can reuse this string allocation but not worth it at all
        debug_assert!(batch.len() <= batch_size);
    }
    if ! batch.is_empty() {
        debug!("handling last batch #{} of size {} (limit {})", batch_nr, batch.len(), batch_size);
        run_batch(&batch, &task, writer).await?;
    }
    Ok(())
}

async fn run_batch(batch: &[String], task: &Task, writer: &mut impl LineWriter) -> Result<(), String> {
    let res = task.execute_with_stdout_nomonitor(writer, &mut StdWriter::stderr()).await;
    Ok(())
}
