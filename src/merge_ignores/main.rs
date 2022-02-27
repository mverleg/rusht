
use ::bump_alloc::BumpAlloc;
use ::gitignore::File;

use ::rusht::make_ignore_walker;
use ::rusht::stop;

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    make_ignore_walker("./").into_iter()
        .filter(|pth|
            pth.file_name()
                .map(|name| name.to_string_lossy())
                .filter(|name| name.starts_with(".") && name.ends_with("ignore"))
                .is_some()
        )
        .map(|pth| match File::new(&pth) {
            Ok(file) => file,
            Err(err) => {
                stop!("failed to parse ignore file: {}", err);
            }
        })
        .collect::<Vec<_>>();
    //TODO @mark:
}
