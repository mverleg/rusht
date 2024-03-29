use ::std::io;
use ::std::time::SystemTime;
use ::std::time::UNIX_EPOCH;

use ::async_std::fs;
use ::log::debug;
use ::sha2::Digest;
use ::sha2::Sha256;
use ::walkdir::DirEntry;
use ::walkdir::WalkDir;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::common::{safe_filename, LineWriter};
use crate::find::jl_args::{ErrorHandling, JlArgs};
use crate::find::jl_json_api::FSNode;
use crate::ExitStatus;

pub async fn list_files(args: JlArgs, writer: &mut impl LineWriter) -> ExitStatus {
    assert!(!(args.no_dirs && args.only_dirs));
    if args.max_depth == 0 {
        eprintln!("jq max-depth is 0, likely should be at least 1")
    }

    let mut has_err = false;
    let mut is_first = true;
    let mut line = String::new();
    if !args.entry_per_lines {
        line.push('[');
    }
    //TODO @mverleg: async walk dir?
    let walker = WalkDir::new(&args.root)
        .max_depth(args.max_depth.try_into().expect("max depth too large"))
        .min_depth(1)
        .follow_links(!args.no_recurse_symlinks);
    let now_ts_sec = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(_) => {
            eprintln!("could not get the current time");
            return ExitStatus::err()
        }
    };
    for entry_res in walker.into_iter() {
        let node = match analyze_file(entry_res, &args, now_ts_sec).await {
            Ok(Some(node)) => node,
            Ok(None) => continue,
            Err(err) => {
                match args.on_error {
                    ErrorHandling::Abort => {
                        eprintln!("{err}");
                        return ExitStatus::of(1);
                    }
                    ErrorHandling::FailAtEnd => {
                        eprintln!("{err}");
                        has_err = true;
                    }
                    ErrorHandling::Warn => eprintln!("{err}"),
                    ErrorHandling::Ignore => debug!("ignoring file read error: {err}"),
                }
                continue;
            }
        };
        if is_first {
            is_first = false;
        } else {
            if !args.entry_per_lines {
                line.push(',');
            }
            writer.write_line(&line).await;
            line.push(' ');
            line.clear();
        }
        line.push_str(&serde_json::to_string(&node).expect("failed to create json from FSNode"));
        // unnecessary allocation but probably not performance-critical ^
    }
    if !args.entry_per_lines {
        line.push(']');
    }
    writer.write_line(&line).await;
    ExitStatus::of_is_ok(!has_err)
}

async fn analyze_file(entry_res: walkdir::Result<DirEntry>, args: &JlArgs, now_ts_sec: u64) -> Result<Option<FSNode>, String> {
    let entry = entry_res.map_err(|err| format!("failed to read file/dir inside {}, err: {err}", args.root.to_string_lossy()))?;
    let path = entry.path();
    let log_path_owned = path.to_string_lossy();
    let log_path = log_path_owned.as_ref();
    let metadata = entry
        .metadata()
        .map_err(|err| format!("could not get metadata for {log_path}, err {err}"))?;

    let name = path
        .file_name()
        .ok_or_else(|| "could not read filename".to_owned())?
        .to_str()
        .ok_or_else(|| "could not convert filename to utf8".to_owned())?;
    if let Some(pattern) = &args.filter {
        if !pattern.is_match(name) {
            return Ok(None);
        }
    }
    let is_dir = metadata.is_dir();
    if args.no_dirs && is_dir {
        return Ok(None);
    }
    if args.only_dirs && !is_dir {
        return Ok(None);
    }
    let filesize_b = metadata.len();
    let filesize_bm = ((filesize_b as f64) / (1024. * 1024.)).round() as u64;
    let hash = if args.do_hash && !is_dir {
        let content: String = fs::read_to_string(path).await.map_err(|err| {
            format!("could not read file content for hashing for {log_path}, err {err}")
        })?;
        Some(compute_hash(&content))
    } else {
        None
    };

    let base_name = path.file_stem()
        .map(|nm| nm.to_str().expect("not utf8")).unwrap_or_else(|| "").to_owned();
    let extension = path.extension()
        .map(|nm| nm.to_str().expect("not utf8")).unwrap_or_else(|| "").to_owned();
    let rel_path = path.strip_prefix(&args.root)
        .map(|pth| pth.to_str().expect("not utf8")).unwrap_or_else(|_| "").to_owned();
    let canonical_path = path.canonicalize()
        .map_err(|err| format!("could not get canonical (abs) path for {log_path}, err {err}"))?
        .to_str().ok_or_else(|| format!("could not convert canonical (abs) path for {log_path} to utf8"))?.to_owned();

    let created_ts = to_timestamp(metadata.created(), log_path)?;
    let changed_ts = to_timestamp(metadata.modified(), log_path)?;
    let changed_age_sec = now_ts_sec.saturating_sub(changed_ts);

    Ok(Some(FSNode {
        name: name.to_owned(),
        safe_name: safe_filename(name),
        base_name,
        extension,
        rel_path,
        canonical_path,
        is_dir,
        is_link: entry.path_is_symlink(),
        size_b: filesize_b,
        size_mb: filesize_bm,
        created_ts,
        //created_by: "".to_owned(),
        changed_ts,
        changed_age_sec,
        //changed_by: "".to_string(),
        hash,
    }))
    //TODO @mverleg: make sure all fields are filled
}

fn to_timestamp(result: io::Result<SystemTime>, log_path: &str) -> Result<u64, String> {
    Ok(result
        .map_err(|err| format!("fould not get created time for {log_path}, err: {err}"))?
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("fould not get created time for {log_path}, err: {err}"))?
        .as_secs())
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash_out = hasher.finalize();
    let encoded = URL_SAFE_NO_PAD.encode(hash_out);
    format!("sha256:{}", encoded.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use ::std::fs;
    use std::path::Path;

    use ::regex::Regex;

    use crate::common::CollectorWriter;

    use super::*;

    #[async_std::test]
    async fn shallow_list_files_per_line_with_hash() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        fs::write(dir_path.join("file1.txt"), "(no content 1)").unwrap();
        fs::write(dir_path.join("file2"), "(no content 2)").unwrap();
        fs::create_dir_all(dir_path.join("subdir")).unwrap();

        let args = JlArgs {
            max_depth: 1,
            no_recurse_symlinks: false,
            entry_per_lines: true,
            filter: None,
            on_error: ErrorHandling::Abort,
            root: dir_path.to_owned(),
            do_hash: true,
            only_dirs: false,
            no_dirs: false,
        };

        let mut writer = CollectorWriter::new();
        let line_container = writer.lines();
        let status = list_files(args, &mut writer).await;
        let lines = line_container.snapshot().await;

        lines
            .iter()
            .enumerate()
            .for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].starts_with('['));
        assert!(!lines[1].ends_with(','));
        assert!(!lines[2].ends_with(']'));
        assert_eq!(
            lines.iter().filter(|l| l.contains("\"file1.txt\"")).count(),
            1
        );
        assert_eq!(lines.iter().filter(|l| l.contains("\"file2\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"subdir\"")).count(), 1);
        assert_eq!(
            lines
                .iter()
                .filter(|l| l.contains("\"safe_name\":\"file1_txt\""))
                .count(),
            1
        );
        assert_eq!(
            lines
                .iter()
                .filter(|l| l.contains("sha256:hmfxpto-b9_cqulnzrwplpjh3mdi7zpitalvzledshe"))
                .count(),
            1
        );
    }

    #[async_std::test]
    async fn deep_filtered_list_only_files_as_json_list() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        setup_fs_for_nested_needle_search(dir_path);

        let args = JlArgs {
            max_depth: 1000,
            no_recurse_symlinks: false,
            entry_per_lines: false,
            filter: Some(Regex::new("^needle.*$").unwrap()),
            on_error: ErrorHandling::FailAtEnd,
            root: dir_path.to_owned(),
            do_hash: false,
            only_dirs: false,
            no_dirs: true,
        };

        let mut writer = CollectorWriter::new();
        let line_container = writer.lines();
        let status = list_files(args, &mut writer).await;
        let lines = line_container.snapshot().await;

        lines
            .iter()
            .enumerate()
            .for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with('['));
        assert!(lines[0].ends_with(','));
        assert!(lines[1].ends_with(']'));
        assert_eq!(
            lines
                .iter()
                .filter(|l| l.contains("do-not-find.txt"))
                .count(),
            0
        );
        assert_eq!(
            lines
                .iter()
                .filter(|l| l.contains("\"needle.txt\""))
                .count(),
            1
        );
        assert!(!lines[1].contains("hash"));
    }

    fn setup_fs_for_nested_needle_search(dir_path: &Path) {
        let sub1 = dir_path.join("subdir");
        let sub2 = dir_path.join("subdir2");
        let sub3 = sub1.join("deeper");
        fs::create_dir_all(&sub1).unwrap();
        fs::create_dir_all(&sub2).unwrap();
        fs::create_dir_all(&sub3).unwrap();
        fs::write(sub1.join("needle.txt"), "(no content)").unwrap();
        fs::write(sub2.join("do-not-find.txt"), "(no content)").unwrap();
        fs::write(sub3.join("needle-name-so-as-to-be-found"), "(no content)").unwrap();
    }

    #[async_std::test]
    async fn ser_deser() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        setup_fs_for_nested_needle_search(dir_path);

        let mut writer = CollectorWriter::new();
        let line_container = writer.lines();
        let status = list_files(
            JlArgs {
                root: dir_path.to_owned(),
                ..JlArgs::default()
            },
            &mut writer,
        )
        .await;
        let lines = line_container.snapshot().await;

        assert!(status.is_ok());
        assert!(lines.len() >= 1);

        let res: Vec<FSNode> = serde_json::from_str(&lines.join("\n")).expect("failed to parse");
        assert_eq!(lines.len(), res.len());
    }
}
