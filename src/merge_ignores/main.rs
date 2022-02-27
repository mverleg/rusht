use ::bump_alloc::BumpAlloc;
use ::ignore::WalkParallel;

use ::rusht::make_ignore_walker;

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    for result in make_ignore_walker("./") {
        println!("file = {:?}", result);
    }
}
