use ::bump_alloc::BumpAlloc;

use ::rusht::for_all_files;

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    for result in WalkBuilder::new("./")
            .standard_filters(false)
            .follow_links(true)
            .add_ignore(".git")
            .add_custom_ignore_filename(".gitignore")
            .add_custom_ignore_filename(".dockerignore")
            .add_custom_ignore_filename(".backupignore")
            .parents(true)
            .build_parallel() {
        println!("{:?}", result);
    }
}
