use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use ::log::debug;
use async_std::stream::Extend;
use itertools::Itertools;
use regex::Regex;

use crate::common::{LineReader, LineWriter, StdWriter, Task};
use crate::textproc::batched_args::BatchedArgs;

pub async fn batched(
    args: BatchedArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> Result<(), String> {
    let batch_size: usize = args.batch_size.try_into().expect("usize too small");
    let task = args.cmd.into_task();
    let grouping = args.together.as_ref().map(|pattern| (Grouping::Together, pattern))
        .or_else(|| args.together.as_ref().map(|pattern| (Grouping::Apart, pattern)));
    if let Some((strategy, pattern)) = grouping {
        batched_filtered_io(task, pattern, strategy, reader, writer, batch_size, args.mixed_groups, args.drop_unmatched).await?;
    } else {
        assert!(!args.mixed_groups, "--mixed-groups not applicable without grouping");
        batched_unfiltered(task, reader, writer, batch_size).await?;
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
    batch_size: usize,
    mixed_groups: bool,
    drop_unmatched: bool,
) -> Result<(), String> {
    let mut lines = Vec::new();
    while let Some(line) = reader.read_line().await {
        lines.push(line.to_owned())
    }
    let (groups, mut remainder) = group_lines_by_regex(lines, pattern);
    let groups = groups.into_iter()
        .map(|(k, v)| v)
        .sorted_by_key(|v| usize::MAX - v.len())
        .collect();
    if drop_unmatched && ! remainder.is_empty() {
        debug!("dropping {} remaining items", remainder.len());
        remainder.clear();

    }
    let batches = match grouping {
        Grouping::Together => batched_together(groups, remainder, batch_size, mixed_groups),
        Grouping::Apart => batched_apart(groups, remainder, batch_size, mixed_groups),
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
            Entry::Vacant(new) => { new.insert(vec![line]); },
        }
    }
    (groups, remainder)
}

fn batched_together(
    groups: Vec<Vec<String>>,
    remainder: Vec<String>,
    batch_size: usize,
    mixed_groups: bool
) -> Vec<Vec<String>> {
    let mut batches = Vec::with_capacity(max(8, groups.len()));
    for mut group in groups {
        group.reverse();
        // If the group is too big for a batch, create new batches
        while group.len() > batch_size {
            let mut batch = Vec::with_capacity(batch_size);
            for _ in 0..batch_size {
                batch.push(group.pop().unwrap());
            }
            batches.push(batch)
        }
        if mixed_groups {
            // Insert into the first group that has enough space
            for batch in &mut batches {
                if group.len() < batch_size - batch.len() {
                    for line in group.drain(..) {
                        batch.push(line);
                    }
                    break
                }
            }
        }
        // Any remainder goes into a new batch
        if ! group.is_empty() {
            group.reverse();
            batches.push(group);
        }
    }
    let mut remainder = remainder;
    remainder.reverse();
    if mixed_groups {
        'outer: for batch in &mut batches {
            while batch.len() < batch_size {
                let Some(line) = remainder.pop() else {
                    break 'outer
                };
                batch.push(line)
            }
        }
    }
    while ! remainder.is_empty() {
        let mut batch = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            let Some(line) = remainder.pop() else {
                break
            };
            batch.push(line)
        }
        batches.push(batch)
    }
    batches
}

fn batched_apart(
    groups: Vec<Vec<String>>,
    remainder: Vec<String>,
    batch_size: usize,
    mixed_groups: bool
) -> Vec<Vec<String>> {
    let capacity = max(remainder.len(), groups.get(0).map(|g| g.len()).unwrap_or(1));
    let mut batches: Vec<Vec<String>> = Vec::with_capacity(capacity);
    'outer: for mut group in groups {
        group.reverse();
        for batch in &mut batches {
            if batch.len() < batch_size {
                let Some(line) = group.pop() else {
                    break 'outer
                };
                batch.push(line)
            }
        }
        while let Some(line) = group.pop() {
            let mut batch = Vec::with_capacity(batch_size);
            batch.push(line);
            batches.push(batch);
        }
    }
    let mut remainder = remainder;
    remainder.reverse();
    if mixed_groups {
        for batch in &mut batches {
            while batch.len() < batch_size {
                let Some(line) = remainder.pop() else {
                    break
                };
                batch.push(line)
            }
        }
    }
    while let Some(line) = remainder.pop() {
        let mut batch = Vec::with_capacity(batch_size);
        while batch.len() < batch_size {
            let Some(line) = remainder.pop() else {
                break
            };
            batch.push(line)
        }
        batches.push(batch);
    }
    batches
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

    #[ignore]  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    #[async_std::test]
    async fn batch_2_wcl() {
        let mut writer = CollectorWriter::new();
        let out_lines = writer.lines();
        let inp = vec!["a", "b", "c", "d", "e"];
        let args = BatchedArgs { cmd: CommandArgs::Cmd(vec!["wc".to_owned(), "-l".to_owned()]), batch_size: 2,
            together: None, apart: None, mixed_groups: false, drop_unmatched: false };
        let res = batched(args, &mut VecReader::new(inp), &mut writer).await;
        assert!(res.is_err());
        assert_eq!(*out_lines.snapshot().await, vec!["2".to_owned(), "2".to_owned(), "1".to_owned()]);
    }

    #[test]
    fn group_by_re() {
        let lines = make_test_lines();
        let re = Regex::new("^\\w+").unwrap();
        let (groups, remainder) = group_lines_by_regex(lines, &re);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("hello").unwrap(), &["hello world".to_owned(), "hello moon".to_owned()]);
        assert_eq!(groups.get("good").unwrap(), &["good night moon".to_owned(), "good".to_owned()]);
        assert_eq!(remainder, vec!["  ".to_owned()]);
    }

    #[test]
    fn group_by_re_group() {
        let lines = make_test_lines();
        let re = Regex::new("^\\w+.* (\\w+)$").unwrap();
        let (groups, remainder) = group_lines_by_regex(lines, &re);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("world").unwrap(), &["hello world".to_owned()]);
        assert_eq!(groups.get("moon").unwrap(), &["hello moon".to_owned(), "good night moon".to_owned(),]);
        assert_eq!(remainder, vec!["good".to_owned(), "  ".to_owned()]);
    }

    fn make_test_lines() -> Vec<String> {
        vec![
            "hello world".to_owned(),
            "hello moon".to_owned(),
            "good night moon".to_owned(),
            "good".to_owned(),
            "  ".to_owned(),
        ]
    }

    #[test]
    fn together_pure() {
        let batches = batched_together(
            vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned(), "e".to_owned(), "f".to_owned()]],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned(), "j".to_owned(), "k".to_owned()],
            3,
            false,
        );
        assert_eq!(batches, vec![
            vec!["a".to_owned(), "b".to_owned()],
            vec!["c".to_owned(), "d".to_owned(), "e".to_owned()],
            vec!["f".to_owned()],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned()],
            vec!["j".to_owned(), "k".to_owned()],
        ]);
    }

    #[test]
    fn together_mixed() {
        let batches = batched_together(
            vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned(), "e".to_owned(), "f".to_owned()]],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned(), "j".to_owned(), "k".to_owned()],
            3,
            true,
        );
        assert_eq!(batches, vec![
            vec!["a".to_owned(), "b".to_owned(), "g".to_owned()],
            vec!["c".to_owned(), "d".to_owned(), "e".to_owned()],
            vec!["f".to_owned(), "h".to_owned(), "i".to_owned()],
            vec!["j".to_owned(), "k".to_owned()],
        ]);
    }

    #[test]
    fn apart_pure() {
        let batches = batched_apart(
            vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned(), "e".to_owned(), "f".to_owned()]],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned(), "j".to_owned(), "k".to_owned()],
            3,
            false,
        );
        assert_eq!(batches, vec![
            vec!["a".to_owned(), "b".to_owned()],
            vec!["c".to_owned(), "d".to_owned(), "e".to_owned()],
            vec!["f".to_owned()],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned()],
            vec!["j".to_owned(), "k".to_owned()],
        ]);
    }

    #[test]
    fn apart_mixed() {
        let batches = batched_apart(
            vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned(), "e".to_owned(), "f".to_owned()]],
            vec!["g".to_owned(), "h".to_owned(), "i".to_owned(), "j".to_owned(), "k".to_owned()],
            3,
            true,
        );
        assert_eq!(batches, vec![
            vec!["a".to_owned(), "b".to_owned(), "g".to_owned()],
            vec!["c".to_owned(), "d".to_owned(), "e".to_owned()],
            vec!["f".to_owned(), "h".to_owned(), "i".to_owned()],
            vec!["j".to_owned(), "k".to_owned()],
        ]);
    }
}
