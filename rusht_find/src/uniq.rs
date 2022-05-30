use ::std::collections::HashSet;

use ::itertools::Itertools;
use ::log::debug;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "uniq_prefix", about = "Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested.")]
pub struct UniqArgs {
    #[structopt(short = "s", long, help = "Sort the entries")]
    pub sorted: bool,
}

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "uniq_prefix", about = "Remove any lines for which any other line is a prefix. E.g. /a and /a/b will remove the latter.")]
pub struct UniqPrefixArgs {
    #[structopt(short = "s", long, help = "Sort the entries")]
    pub sorted: bool,
}

fn uniq<S>(args: &UniqPrefixArgs, texts: &[S]) -> Vec<String>
    where S: AsRef<str>, S: Into<String> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
        if ! seen.insert(txt) {
            eprintln!("  DUPLICATE {}", txt);  //TODO @mark:
            continue
        }
        result.push(txt.into())
    }
    if args.sorted {
        debug!("sorting uniq_prefix result");
        result.sort_unstable()
    }
    result
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
fn uniq_prefix<S>(args: &UniqPrefixArgs, texts: &[S]) -> Vec<String>
    where S: AsRef<str>, S: Into<String> {
    let known: uniq(&UniqArgs {
        sorted: true,
    }, texts);
    debug!("finding uniq_prefix in {} items", known.len());
    dbg!(&known);  //TODO @mark: TEMPORARY! REMOVE THIS!
    let mut result = Vec::with_capacity(known.len());
    let mut seen = HashSet::with_capacity(known.len());
    for txt in texts {
        let txt = txt.as_ref();
        if ! seen.insert(txt) {
            eprintln!("  DUPLICATE {}", txt);  //TODO @mark:
            continue
        }
        let indx = known.binary_search(&txt)
            .expect("should always be found since collections have the same elements");
        eprintln!(" 0: compare {} to {}", txt, &texts[indx].as_ref());  //TODO @mark:
        if indx > 0 {
            let other = texts[indx - 1].as_ref();
            eprintln!("-1: compare {} to {}", txt, other);  //TODO @mark:
            if txt.starts_with(other) {
                eprintln!("  DROP {}", txt);  //TODO @mark:
                continue
            }
        }
        if indx + 1 < texts.len() {
            eprintln!("+1: compare {} to {}", txt, &texts[indx + 1].as_ref());  //TODO @mark:
        }
        result.push(txt.into())
    }
    if args.sorted {
        debug!("sorting uniq_prefix result");
        result.sort_unstable()
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniq_prefix_first() {
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a", "/a/b", "/a/c"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_duplicates() {
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a", "/a", "/a"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_middle() {
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a/c", "/a", "/a/b"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_nomatch() {
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a/c", "/a/b"]);
        assert_eq!(res, vec!["/a/c".to_owned(), "/a/b".to_owned()]);
    }
}
