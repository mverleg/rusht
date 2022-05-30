use ::itertools::Itertools;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "uniq_prefix", about = "Remove ann lines for which any other line is a prefix. E.g. /a and /a/b will remove the latter.")]
pub struct UniqPrefixArgs {
    #[structopt(short = "s", long, help = "Sort the entries")]
    pub sorted: bool,
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
fn uniq_prefix<S>(args: &UniqPrefixArgs, texts: &[S]) -> Vec<String>
        where S: AsRef<str>, S: Into<String> {
    let known: Vec<&str> = texts.iter()
        .map(|txt| txt.as_ref())
        .sorted()
        .collect();
    let mut result = Vec::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
        let indx = known.binary_search(&txt)
            .expect("should always be found since collections have the same elements");
        eprintln!("compare {} to {}", txt, &texts[indx].as_ref());  //TODO @mark:
        result.push(txt.into())
    }
    if args.sorted {
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
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a/b", "/a", "/a/c"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_nomatch() {
        let res = uniq_prefix(&UniqPrefixArgs::default(), &vec!["/a/b", "/a/c"]);
        assert_eq!(res, vec!["/a/b".to_owned(), "/a/c".to_owned()]);
    }
}
