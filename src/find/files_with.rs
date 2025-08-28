use ::std::fs;
use ::std::io::Read;
use ::std::path::Path;

use crate::common::LineWriter;
use crate::find::files_with_args::FilesWithArgs;
use crate::find::files_with_args::OnErr;
use crate::ExitStatus;
use ::ignore::WalkBuilder;
use ::log::debug;
use ::log::warn;

pub async fn files_with(
        args: FilesWithArgs,
        writer: &mut impl LineWriter
) -> ExitStatus {
    let mut non_utf8_count = 0;

    for root in &args.roots {
        match find_files_with(&args, root, writer, &mut non_utf8_count).await {
            Ok(()) => {},
            Err(err) => {
                eprintln!("Error processing root '{}': {}", root.display(), err);
                return ExitStatus::err();
            }
        }
    }

    if non_utf8_count > 0 {
        eprintln!("Skipped {} non-UTF8 files", non_utf8_count);
    }

    ExitStatus::ok()
}

async fn find_files_with(args: &FilesWithArgs, root: &Path, writer: &mut impl LineWriter, non_utf8_count: &mut u32) -> Result<(), String> {
    let mut walker_builder = WalkBuilder::new(root);
    walker_builder
        .max_depth(Some(args.max_depth as usize));
    
    // Configure gitignore handling

    if let Some(use_gitignore) = Some(true) {
        walker_builder
            .git_ignore(use_gitignore)
            .git_global(use_gitignore)
            .git_exclude(use_gitignore);
    }
    let walker = walker_builder.build();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                match args.on_err {
                    OnErr::Warn => {
                        warn!("Error walking directory: {}", err);
                        continue;
                    }
                    OnErr::Abort => {
                        return Err(format!("Error walking directory: {}", err));
                    }
                    OnErr::Ignore => {
                        debug!("Ignoring walk error: {}", err);
                        continue;
                    }
                }
            }
        };

        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let path = entry.path();
            
            if file_matches(args, path, non_utf8_count)? {
                // Write result immediately - always use canonical path
                let path_str = path.to_string_lossy();
                writer.write_line(&path_str).await;
            }
        }
    }

    Ok(())
}

fn file_matches(args: &FilesWithArgs, path: &Path, non_utf8_count: &mut u32) -> Result<bool, String> {
    // Check path patterns
    if let Some(path_str) = path.to_str() {
        for pattern in &args.paths {
            if !pattern.is_match(path_str) {
                return Ok(false);
            }
        }
        for pattern in &args.not_paths {
            if pattern.is_match(path_str) {
                return Ok(false);
            }
        }
    }

    // Check content patterns (only if there are content patterns to check)
    if !args.contents.is_empty() || !args.not_contents.is_empty() {
        let content = match read_file_content(path) {
            Ok(content) => content,
            Err(_) => {
                // Skip non-UTF8 files and increment counter
                *non_utf8_count += 1;
                return Ok(false);
            }
        };
        
        for pattern in &args.contents {
            if !pattern.is_match(&content) {
                return Ok(false);
            }
        }
        for pattern in &args.not_contents {
            if pattern.is_match(&content) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn read_file_content(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|err| format!("Failed to open file {}: {}", path.display(), err))?;
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|err| format!("Failed to read file {}: {}", path.display(), err))?;
    
    Ok(content)
}