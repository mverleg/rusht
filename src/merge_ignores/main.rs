use ::std::fs::read_to_string;
use ::std::path::Path;
use std::path::PathBuf;

use ::bump_alloc::BumpAlloc;
use ::gitignore::Pattern;
use ::lazy_static::lazy_static;
use ::regex::Regex;

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
    let ignores = collect_ignore_paths("./");
    let _ignore_patterns = find_ignore_patterns(&ignores);
    //TODO @mark:
}

pub fn collect_ignore_paths(pth: &str) -> Vec<PathBuf> {
    return make_ignore_walker(pth).into_iter()
        .filter(|pth| pth.file_name()
            .map(|name| name.to_string_lossy())
            .filter(|name| name.starts_with(".") && name.ends_with("ignore"))
            .is_some())
        .collect::<Vec<_>>()
}

fn find_ignore_patterns(paths: &[PathBuf]) -> Vec<Pattern> {
    let mut patterns = vec![];
    for pth in paths {
        let content = read_to_string(&pth)
            .unwrap_or_else(|err| stop!("failed to read ignore file; err: {}", err));
        for line in content.lines() {
            if !PATTERN_RE.is_match(line) {
                continue;
            }
            patterns.push(parse_pattern(line, pth.as_path()));
        }
    }
    patterns
}

fn parse_pattern<'a>(line: &str, ignore_pth: &'a Path) -> Pattern<'a> {
    let root_pth = ignore_pth.canonicalize().unwrap().parent().unwrap();
    match Pattern::new(line, root_pth) {
        Ok(pattern) => pattern,
        Err(err) => stop!("failed to parse pattern '{}', err: {}", line, err),
    }
}
