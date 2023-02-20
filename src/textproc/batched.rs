use std::collections::hash_map::Entry;
use std::collections::HashMap;
use ::log::debug;
use regex::{Match, Regex};

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
    let groups = group_lines_by_regex(lines, pattern);
    let batches = match grouping {
        Grouping::Together => batched_together(groups.0, groups.1, batch_size),
        Grouping::Apart => batched_apart(groups.0, groups.1, batch_size),
    };
    for (batch_nr, batch) in batches.into_iter().enumerate() {
        debug!("handling batch #{} of size {}, grouped {:?} by {}", batch_nr, batch.len(), grouping, pattern);
        run_batch(&batch, &task, writer).await?;
    }
    Ok(())
}

fn group_lines_by_regex(
    lines: Vec<String>,
    pattern: &Regex,
) -> (HashMap<String, Vec<String>>, Vec<String>) {
    let mut has_warned = false;
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let mut remainder: Vec<String> = Vec::new();
    for line in lines {
        let mut iter = pattern.captures(&line).into_iter();
        let matches = iter.next()
            .map(|first_pattern_match| (
                first_pattern_match.get(0).unwrap(),
                first_pattern_match.get(1),
                first_pattern_match.get(2),));
        if !has_warned && iter.next().is_some() {
            eprintln!("batched: pattern matched more than once, only first result is used");
            has_warned = true
        }
        let Some(matc) = matches else {
            // No match, do not group line
            remainder.push(line);
            continue
        };
        let group = match matc {
            (_, _, Some(_)) => {
                panic!("batched: more than one capture group in the pattern, only one group can be captured")
            }
            (_, Some(first_group), None) => {
                // Use first capture group
                first_group.as_str().to_owned()
            }
            (full, None, None) => {
                // Probably no groups, use full pattern
                full.as_str().to_owned()
            }
        };
        match groups.entry(group) {
            Entry::Occupied(mut existing) => existing.get_mut().push(line),
            Entry::Vacant(mut new) => new.insert(vec![line]),
        }
        // let Some(matches) =  pattern.captures(&line) else {
        //     // not matched, add complete line to remainder
        //     remainder.push(line);
        //     continue
        // };
        // let key = matches.iter().next().expect("first group is full match, exists")
        // for matc in matches.iter().take(2) {
        //
        // }
        // let Some(first_match) = matches.iter().next() else {
        //     // not groups, use full regex match as a group
        //     groups.entry(matches.unwrap().unwrap().as_str().to_owned());
        //     continue
        // };
        //
        // for captures in pattern.captures(&line) {
        //     any_matches = true;
        //     let mut caps = captures.iter();
        //     let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
        //     let mut any_groups = false;
        //     // Within a pattern match, iterate over the capture groups
        //     for mtch_opt in caps {
        //         any_groups = true;
        //         if let Some(mtch) = mtch_opt {
        //             writer.write_line(mtch.as_str()).await;
        //             match_cnt += 1
        //         }
        //         if first_capture_only {
        //             break;
        //         }
        //     }
        //     if !any_groups {
        //         writer.write_line(full_match).await;
        //     }
        //     if first_match_only {
        //         break;
        //     }
        // }
    }
    (groups, remainder)
}

fn batched_together(
    groups: HashMap<String, Vec<String>>,
    remainder: Vec<String>,
    batch_size: usize
) -> Vec<Vec<String>> {
    todo!();  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}

fn batched_apart(
    groups: HashMap<String, Vec<String>>,
    remainder: Vec<String>,
    batch_size: usize
) -> Vec<Vec<String>> {
    todo!();  //TODO @mverleg: TEMPORARY! REMOVE THIS!
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
