use ::ignore::WalkBuilder;

pub fn make_ignore_walker() {
    WalkBuilder::new("./")
        .standard_filters(false)
        .follow_links(true)
        .add_ignore(".git")
        .add_custom_ignore_filename(".gitignore")
        .add_custom_ignore_filename(".dockerignore")
        .add_custom_ignore_filename(".backupignore")
        .parents(true)
        .build_parallel()
}