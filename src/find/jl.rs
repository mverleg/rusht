use ::std::io;
use ::std::time::{SystemTime, UNIX_EPOCH};

use ::async_std::fs;
use ::log::debug;
use ::sha2::Digest;
use ::sha2::Sha256;
use ::walkdir::DirEntry;
use ::walkdir::WalkDir;

use crate::common::{LineWriter, safe_filename};
use crate::ExitStatus;
use crate::find::jl_args::{ErrorHandling, JlArgs};
use crate::find::jl_json_api::FSNode;

pub async fn list_files(
    args: JlArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!(args.no_dirs && args.only_dirs));
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
    let walker = WalkDir::new(&args.root)
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
            line.clear();
        }
        let node = match analyze_file(entry_res, &args).await {
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
        // unnecessary allocation but probably not performance-critical ^
    }
    if ! args.entry_per_lines {
        line.push(']');
    }
    writer.write_line(&line).await;
    assert!(!has_err);  //TODO @mverleg: msg
    ExitStatus::ok()
}

async fn analyze_file(entry_res: walkdir::Result<DirEntry>, args: &JlArgs) -> Result<Option<FSNode>, String> {
    let entry = entry_res.map_err(|err| format!("failed to read file/dir, err: {err}"))?;
    let path = entry.path();
    let log_path_owned = path.to_string_lossy();
    let log_path = log_path_owned.as_ref();
    let metadata = entry.metadata().map_err(|err| format!("could not get metadata for {log_path}, err {err}"))?;

    let name = path.file_name()
        .ok_or_else(|| "could not read filename".to_owned())?
        .to_str()
        .ok_or_else(|| "could not convert filename to utf8".to_owned())?;
    if let Some(pattern) = &args.filter {
        if ! pattern.is_match(name) {
            return Ok(None)
        }
    }
    let is_dir = metadata.is_dir();
    if args.no_dirs && is_dir {
        return Ok(None)
    }
    if args.only_dirs && ! is_dir {
        return Ok(None)
    }
    let filesize_b = metadata.len();
    let filesize_bm = ((filesize_b as f64) / (1024. * 1024.)).round() as u64;
    let hash = if args.do_hash && ! is_dir {
        let content: String = fs::read_to_string(path).await
            .map_err(|err| format!("could not read file content for hashing for {log_path}, err {err}"))?;
        Some(compute_hash(&content))
    } else {
        None
    };

    Ok(Some(FSNode {
        name: name.to_owned(),
        safe_name: safe_filename(name),
        base_name: "".to_string(),
        extension: "".to_string(),
        rel_path: "".to_string(),
        canonical_path: path.canonicalize()
            .map_err(|err| format!("could not get canonical (abs) path for {log_path}, err {err}"))?
            .to_str().ok_or_else(|| format!("could not convert canonical (abs) path for {log_path} to utf8"))?.to_owned(),
        is_dir,
        is_link: entry.path_is_symlink(),
        size_b: filesize_b,
        size_mb: filesize_bm,
        created_ts: to_timestamp(metadata.created(), log_path)?,
        created_by: "".to_owned(),
        changed_ts: to_timestamp(metadata.modified(), log_path)?,
        changed_age_sec: "".to_string(),
        changed_by: "".to_string(),
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

const URL_SAFE_NO_PAD: base64::engine::fast_portable::FastPortable =
    base64::engine::fast_portable::FastPortable::from(
        &base64::alphabet::URL_SAFE,
        base64::engine::fast_portable::NO_PAD,
    );

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash_out = hasher.finalize();
    let encoded = base64::encode_engine(hash_out, &URL_SAFE_NO_PAD);
    format!("sha256:{}", encoded.to_ascii_lowercase())
}


#[cfg(test)]
mod tests {
    use ::std::fs;

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

        lines.iter().enumerate().for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].starts_with('['));
        assert!(!lines[1].ends_with(','));
        assert!(!lines[2].ends_with(']'));
        assert_eq!(lines.iter().filter(|l| l.contains("\"file1.txt\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"file2\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"subdir\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"safe_name\":\"file1_txt\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("sha256:hmfxpto-b9_cqulnzrwplpjh3mdi7zpitalvzledshe")).count(), 1);
    }

    #[async_std::test]
    async fn deep_filtered_list_only_files_as_json_list() {
        let dir_handle = tempfile::tempdir().unwrap();
        let dir_path = dir_handle.path();
        let sub1 = dir_path.join("subdir");
        let sub2 = dir_path.join("subdir2");
        let sub3 = sub1.join("deeper");
        fs::create_dir_all(&sub1).unwrap();
        fs::create_dir_all(&sub2).unwrap();
        fs::create_dir_all(&sub3).unwrap();
        fs::write(sub1.join("needle.txt"), "(no content)").unwrap();
        fs::write(sub2.join("do-not-find.txt"), "(no content)").unwrap();
        fs::write(sub3.join("needle-name-so-as-to-be-found"), "(no content)").unwrap();
        //TODO @mverleg: create deeper directories and mismatching files

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

        lines.iter().enumerate().for_each(|(i, l)| println!("{i}: |{l}|"));
        assert!(status.is_ok());
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with('['));
        assert!(lines[1].ends_with(','));
        assert!(lines[2].ends_with(']'));
        assert_eq!(lines.iter().filter(|l| l.contains("\"file1.txt\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"file2\"")).count(), 1);
        assert_eq!(lines.iter().filter(|l| l.contains("\"subdir\"")).count(), 1);
        assert!(!lines[1].contains("hash"));
    }
}
