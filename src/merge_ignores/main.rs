use ::std::path::PathBuf;
use std::fs::read_to_string;

use ::bump_alloc::BumpAlloc;
use ::gitignore::Pattern;
use ::lazy_static::lazy_static;
use ::regex::Regex;
use gitignore::Error;
use itertools::Itertools;

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
    let ignore_patterns = find_ignore_patterns("./");
    //TODO @mark:
}

fn find_ignore_patterns(pth: &str) -> Vec<PathBuf> {
    make_ignore_walker(pth).into_iter()
        .filter(|pth|
            pth.file_name()
                .map(|name| name.to_string_lossy())
                .filter(|name| name.starts_with(".") && name.ends_with("ignore"))
                .is_some()
        )
        .map(|pth| read_to_string(pth).unwrap_or_else(|err| stop!("failed to read ignore file; err: {}", err)))
        .flat_map(|content| content.lines())
        .filter(|line| PATTERN_RE.is_match(line))
        .map(|line| parse_pattern(line))
        .collect::<Vec<_>>()
}

fn parse_pattern(line: &str) -> Pattern {
    match Pattern::new(line) {
        Ok(pattern) => pattern,
        Err(err) => stop!("failed to parse pattern '{}', err: {}", line, err),
    }
}

// fn parse_patterns(ignore_files: &[PathBuf]) -> Vec<Pattern> {
//     let mut patterns = vec![];
//     for file in ignore_files {
//         //TODO @mark: don't unwrap?
//         let name = file.file_name().unwrap().to_string_lossy().as_ref();
//         if ! &*PATTERN_RE.is_match(name) {
//             continue
//         }
//         let is_neg = &*NEG_PATTERN_RE.is_match(name);
//         unimplemented!("prefix the current directory")
//     }
//     patterns
// }
