use ::std::path::Path;
use std::path::PathBuf;

use ::ignore::WalkBuilder;

pub fn make_ignore_walker(pth: impl AsRef<Path>) -> Vec<PathBuf> {
    WalkBuilder::new(pth.as_ref())
        .standard_filters(false)
        .follow_links(true)
        //.add_ignore(".git")
        .add_custom_ignore_filename(".gitignore")
        .add_custom_ignore_filename(".dockerignore")
        .add_custom_ignore_filename(".backupignore")
        .parents(true)
        .build()
        .into_iter()
        .map(|pth| pth.unwrap().path().to_path_buf())
        .collect::<Vec<_>>()
}
