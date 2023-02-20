use ::log::debug;
use regex::Regex;

use crate::common::{LineReader, LineWriter, StdWriter, Task};
use crate::textproc::batched_args::BatchedArgs;

pub async fn batched(
    args: BatchedArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> Result<(), String> {
    if let Some(_) = args.together {
        unimplemented!("--together not supported")
    }
    if let Some(_) = args.apart {
        unimplemented!("--apart not supported")
    }
    let batch_size: usize = args.batch_size.try_into().expect("usize too small");
    let task = args.cmd.into_task();
    let grouping = args.together.as_ref().map(|pattern| (Grouping::Together, pattern))
        .or_else(|| args.together.as_ref().map(|pattern| (Grouping::Apart, pattern)));
    if let Some((strategy, pattern)) = grouping {
        batched_filtered_io(task, pattern, strategy, reader, writer, batch_size).await;
    } else {
        batched_unfiltered(task, reader, writer, batch_size).await;
    }
    Ok(())
}

async fn batched_unfiltered(
    task: Task, reader:
    &mut impl LineReader, writer:
    &mut impl LineWriter,
    batch_size: usize
) -> Result<(), String> {
    let mut batch = Vec::with_capacity(batch_size);
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
    if !batch.is_empty() {
        debug!("handling last batch #{} of size {} (limit {})", batch_nr, batch.len(), batch_size);
        run_batch(&batch, &task, writer).await?;
    }
    Ok(())
}

#[derive(Debug)]
enum Grouping { Together, Apart }

async fn batched_filtered_io(
    task: Task, pattern: &Regex,
    grouping: Grouping,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
    batch_size: usize
) -> Result<(), String> {
    let mut lines = Vec::new();
    while let Some(line) = reader.read_line().await {
        lines.push(line.to_owned())
    }
    let batches = batched_filtered(lines, pattern, grouping, batch_size);
    for (batch_nr, batch) in batches.into_iter().enumerate() {
        debug!("handling batch #{} of size {}, grouped {:?} by {}", batch_nr, batch.len(), grouping, pattern);
        run_batch(&batch, &task, writer).await?;
    }
    Ok(())
}

fn batched_filtered(
    lines: Vec<String>,
    pattern: &Regex,
    grouping: Grouping,
    batch_size: usize
) -> Vec<Vec<String>> {
    todo!();  //TODO @mverleg: TEMPORARY! REMOVE THIS!

    // let mut batch = Vec::with_capacity(batch_size);
    // let mut batch_nr = 0;
    // while let Some(line) = reader.read_line().await {
    //     if batch.len() >= batch_size {
    //         debug!("handling batch #{} of size {}", batch_nr, batch.len());
    //         batch_nr += 1;
    //         run_batch(&batch, &task, writer).await?;
    //     }
    //     batch.push(line.to_owned());
    //     // ^ can reuse this string allocation but not worth it at all
    //     debug_assert!(batch.len() <= batch_size);
    // }
    // if !batch.is_empty() {
    //     debug!("handling last batch #{} of size {} (limit {})", batch_nr, batch.len(), batch_size);
    //     run_batch(&batch, &task, writer).await?;
    // }
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
        let args = BatchedArgs { batch_size: 2, together: None, apart: None, cmd: CommandArgs::Cmd(vec!["wc".to_owned(), "-l".to_owned()]) };
        let res = batched(args, &mut VecReader::new(inp), &mut writer).await;
        assert!(res.is_err());
        assert_eq!(*out_lines.snapshot().await, vec!["2".to_owned(), "2".to_owned(), "1".to_owned()]);
    }
}
