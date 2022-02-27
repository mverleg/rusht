extern crate bump_alloc;

use bump_alloc::BumpAlloc;

#[global_allocator]
static A : BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

fn main() {
    println!("Hello, world!");
}
