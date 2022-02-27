use std::path::PathBuf;
use ::bump_alloc::BumpAlloc;
use ::gitignore::File;
use ::gitignore::Pattern;
use ::lazy_static::lazy_static;

use ::rusht::make_ignore_walker;
use ::rusht::stop;

lazy_static! {
    static ref PATTERN_RE: Regex = Regex::new(r"^\s*(#.*)*$").unwrap();
    static ref NEG_PATTERN_RE: Regex = Regex::new(r"^\s*!").unwrap();
}

#[global_allocator]
static A: BumpAlloc = BumpAlloc::with_size(1024 * 1024 * 4);

#[tokio::main]
async fn main() {
    let ignore_files = make_ignore_walker("./").into_iter()
        .filter(|pth|
            pth.file_name()
                .map(|name| name.to_string_lossy())
                .filter(|name| name.starts_with(".") && name.ends_with("ignore"))
                .is_some()
        )
        .collect::<Vec<_>>();
    let patterns = parse_patterns(&ignore_files);
    //TODO @mark:
}

fn parse_patterns(ignore_files: &[PathBuf]) -> Vec<Pattern> {
    let mut patterns = vec![];
    for file in ignore_files {
        if ! &*PATTERN_RE.is_match(file) {
            continue
        }
        let is_neg = &*NEG_PATTERN_RE.is_match(file);
        unimplemented!("prefix the current directory")
    }
    patterns
}
