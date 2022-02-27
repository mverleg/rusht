use ::bump_alloc::BumpAlloc;

use ::rusht::for_all_files;

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    for_all_files(|pth| {
        println!("file = {}", pth.to_string_lossy());
        if 1 > 2 { return Err(()) };  //TODO @mark: TEMPORARY! REMOVE THIS!
        Ok(())
    }).await.unwrap();
}
