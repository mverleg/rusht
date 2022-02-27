extern crate bump_alloc;

use ::async_walkdir::WalkDir;
use ::bump_alloc::BumpAlloc;

use ::futures_lite::stream::StreamExt;

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut entries = WalkDir::new(".");
    loop {
        match entries.next().await {
            Some(Ok(entry)) => println!("file: {}", entry.path().display()),
            Some(Err(err)) => panic!("walkdir error: {}", err),
            None => break,
        }
    }
}
