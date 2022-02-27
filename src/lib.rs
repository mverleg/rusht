use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::exit;

use ::ignore::WalkBuilder;

pub fn make_ignore_walker(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let paths = WalkBuilder::new(root.as_ref())
        .standard_filters(false)
        .follow_links(true)
        .add_custom_ignore_filename(".gitignore")
        .add_custom_ignore_filename(".dockerignore")
        .add_custom_ignore_filename(".backupignore")
        .filter_entry(|entry| entry.path().file_name().map(|n| n != ".git").unwrap_or(false))
        .parents(true)
        .build()
        .into_iter()
        .map(|pth| pth.map(|p| p.path().to_path_buf()))
        .filter(|pth| pth.as_ref().map(|p| p.is_file()).unwrap_or(false))
        .collect::<Result<Vec<_>, _>>();
    match paths {
        Ok(paths) => {
            if paths.is_empty() {
                eprintln!("did not find any file that is not ignored in '{}'", root.as_ref().to_string_lossy());
                exit(1)
            }
            paths
        },
        Err(err) => {
            eprintln!("failed to find files: {}", err);
            exit(1)
        }
    }
}
