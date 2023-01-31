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
    todo!("waiting for exec2 code");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::common::{CollectorWriter, CommandArgs, VecReader};
    use super::*;

    #[async_std::test]
    async fn batch_2_wcl() {
        let mut writer = CollectorWriter::new();
        let out_lines = writer.lines();
        let inp = vec!["a", "b", "c", "d", "e"];
        let args = BatchedArgs { batch_size: 2, cmd: CommandArgs::Cmd(vec!["wc".to_owned(), "-l".to_owned()]) };
        let res = batched(args, &mut VecReader::new(inp), &mut writer).await;
        assert!(res.is_err());
        assert_eq!(*out_lines.snapshot().await, vec!["2".to_owned(), "2".to_owned(), "1".to_owned()]);
    }
}
