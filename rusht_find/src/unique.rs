use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "uniq_prefix", about = "Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested.")]
pub struct UniqueArgs {
    #[structopt(short = "s", long, help = "Sort the entries")]
    pub sorted: bool,
    #[structopt(short = "p", long, help = "Remove any lines for which any other line is a prefix. E.g. /a and /a/b will remove the latter.")]
    pub prefix: bool,
    #[structopt(parse(from_flag = Keep::from_find_duplicates), short = "d", long = "find-duplicates", help = "Invert the behaviour, returning all first occurrences and keeping any subsequent duplicates.", conflicts_with="prefix", )]
    pub keep: Keep,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Keep {
    #[default]
    First,
    Subsequent,
    All,
}

impl Keep {
    fn keep_is_first(&self, is_first: bool) -> bool {
        match self {
            Keep::First => is_first,
            Keep::Subsequent => !is_first,
            Keep::All => true,
        }
    }

    fn from_find_duplicates(is_find_duplicates: bool) -> Keep {
        if is_find_duplicates {
            Keep::Subsequent
        } else {
            Keep::First
        }
    }
}

pub fn unique<S>(texts: &[S], sorted: bool, keep: Keep) -> Vec<String>
        where S: AsRef<str>, S: Into<String> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
        if keep.keep_is_first(seen.insert(txt)) {
            eprintln!("  DUPLICATE {}", txt);  //TODO @mark:
            continue
        }
        result.push(txt.into())
    }
    if sorted {
        debug!("sorting uniq_prefix result");
        result.sort_unstable()
    }
    result
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
pub fn unique_prefix<S>(texts: &[S], sorted: bool) -> Vec<String>
    where S: AsRef<str>, S: Into<String> {
    let known = unique(texts, true, Keep::First);
    let known = known.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
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
    if sorted {
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
        let res = unique_prefix(&vec!["/a", "/a/b", "/a/c"], false);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_duplicates() {
        let res = unique_prefix(&vec!["/a", "/a", "/a"], false);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_middle() {
        let res = unique_prefix(&vec!["/a/c", "/a", "/a/b"], false);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_nomatch() {
        let res = unique_prefix(&vec!["/a/c", "/a/b"], false);
        assert_eq!(res, vec!["/a/c".to_owned(), "/a/b".to_owned()]);
    }

    //TODO @mark: uniq

    //TODO @mark: sort
}
