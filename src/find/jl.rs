use ::log::debug;
use ::regex::Regex;
use ::walkdir::DirEntry;
use ::walkdir::WalkDir;

use crate::common::LineWriter;
use crate::ExitStatus;
use crate::find::jl_args::{ErrorHandling, JlArgs};
use crate::find::jl_json_api::FSNode;

pub async fn list_files(
    args: JlArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    if args.max_depth == 0 {
        eprintln!("jq max-depth is 0, likely should be at least 1")
    }

    //TODO @mverleg: filter
    //TODO @mverleg: root
    let mut has_err = false;
    let mut is_first = true;  //TODO @mverleg:
    let mut line = String::new();
    if ! args.entry_per_lines {
        line.push('[');
    }
    //TODO @mverleg: async walk dir?
    let walker = WalkDir::new(args.root)
        .max_depth(args.max_depth.try_into().expect("max depth too large"))
        .min_depth(1)
        .follow_links(!args.no_recurse_symlinks);
    for entry_res in walker.into_iter() {
        if is_first {
            is_first = false;
        } else {
            if ! args.entry_per_lines {
                line.push(',');
            }
            writer.write_line(&line).await;
            eprintln!("line = {}", &line);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
            line.clear();
        }
        let node = match analyze_file(entry_res, &args.filter) {
            Ok(Some(node)) => node,
            Ok(None) => continue,
            Err(err) => {
                match args.on_error {
                    ErrorHandling::Abort => {
                        eprintln!("failed to read file, error: {err}");
                        return ExitStatus::of(1)
                    },
                    ErrorHandling::FailAtEnd => { has_err = true; }
                    ErrorHandling::Warn => eprintln!("failed to read file, error: {err}"),
                    ErrorHandling::Ignore => debug!("ignoring file read error: {err}"),
                }
                continue
            }
        };
        line.push_str(&serde_json::to_string(&node).expect("failed to create json from FSNode"));
        // unnecessary allocation but not performance-critical ^
    }
    if ! args.entry_per_lines {
        line.push(']');
    }
    writer.write_line(&line).await;
    eprintln!("last line = {}", &line);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    assert!(!has_err);  //TODO @mverleg: msg
    ExitStatus::ok()
}

fn analyze_file(entry_res: walkdir::Result<DirEntry>, pattern: &Option<Regex>) -> Result<Option<FSNode>, String> {
    let entry = entry_res.map_err(|err| format!("failed to read file/dir, err: {err}"))?;
    let path = entry.path();
    let log_path_owned = path.to_string_lossy();
    let log_path = log_path_owned.as_ref();
    let name = path.file_name()
        .ok_or_else(|| "could not read filename".to_owned())?
        .to_str()
        .ok_or_else(|| "could not convert filename to utf8".to_owned())?;
    if let Some(pattern) = pattern {
        if ! pattern.is_match(name) {
            return Ok(None)
        }
    }
    Ok(Some(FSNode {
        name: name.to_owned(),
        base_name: "".to_string(),
        extension: "".to_string(),
        rel_path: "".to_string(),
        canonical_path: path.canonicalize()
            .map_err(|err| format!("could not get canonical (abs) path for {log_path}, err {err}"))?
            .to_str().ok_or_else(|| format!("could not convert canonical (abs) path for {log_path} to utf8"))?.to_owned(),
        is_dir: false,
        is_link: false,
        created_ts: (),
        created_by: "".to_string(),
        changed_ts: (),
        changed_age_sec: "".to_string(),
        changed_by: "".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use ::std::fs;

    use ::regex::Regex;

    use crate::common::CollectorWriter;

    use super::*;

    #[async_std::test]
    async fn shallow_list_files_per_line() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        fs::write(dir_path.join("file1.txt"), "(no content)").unwrap();
        fs::write(dir_path.join("file2"), "(no content)").unwrap();
        fs::create_dir_all(dir_path.join("subdir")).unwrap();

        let args = JlArgs {
            max_depth: 1,
            no_recurse_symlinks: false,
            entry_per_lines: true,
            filter: None,
            on_error: ErrorHandling::Abort,
            root: dir_path.to_owned(),
        };

        let mut writer = CollectorWriter::new();
        let line_container = writer.lines();
        let status = list_files(args, &mut writer).await;
        let lines = line_container.snapshot().await;

        lines.iter().enumerate().for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].starts_with('['));
        assert!(!lines[1].ends_with(','));
        assert!(!lines[2].ends_with(']'));
        assert_eq!(lines.iter().filter(|l| l.contains("\"file1.txt\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"file2\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"subdir\"")).count(), 1);
    }

    #[async_std::test]
    async fn deep_filtered_list_files_as_json_list() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        fs::write(dir_path.join("file1.txt"), "(no content)").unwrap();
        fs::write(dir_path.join("file2"), "(no content)").unwrap();
        fs::create_dir_all(dir_path.join("subdir")).unwrap();
        //TODO @mverleg: create deeper directories and mismatching files

        let args = JlArgs {
            max_depth: 1000,
            no_recurse_symlinks: false,
            entry_per_lines: false,
            filter: Some(Regex::new("^needle.*$").unwrap()),
            on_error: ErrorHandling::FailAtEnd,
            root: dir_path.to_owned(),
        };

        let mut writer = CollectorWriter::new();
        let line_container = writer.lines();
        let status = list_files(args, &mut writer).await;
        let lines = line_container.snapshot().await;

        lines.iter().enumerate().for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with('['));
        assert!(lines[1].ends_with(','));
        assert!(lines[2].ends_with(']'));
        assert_eq!(lines.iter().filter(|l| l.contains("\"file1.txt\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"file2\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"subdir\"")).count(), 1);
    }
}
