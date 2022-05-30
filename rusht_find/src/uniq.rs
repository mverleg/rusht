use itertools::Itertools;

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
fn uniq_prefix<S>(texts: &[S]) -> Vec<String> where S: AsRef<str> {
    let known: Vec<&str> = texts.iter()
        .map(|txt| txt.as_ref())
        .sorted()
        .collect();
    let result = Vec::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
        let indx = known.binary_search(&txt)
            .expect("should always be found since collections have the same elements");

    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniq_prefix_first() {
        let res = uniq_prefix(&vec!["/a", "/a/b", "/a/c"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_duplicate() {
        let res = uniq_prefix(&vec!["/a", "/a", "/a"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_middle() {
        let res = uniq_prefix(&vec!["/a/b", "/a", "/a/c"]);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn uniq_prefix_nomatch() {
        let res = uniq_prefix(&vec!["/a/b", "/a/c"]);
        assert_eq!(res, vec!["/a/b".to_owned(), "/a/c".to_owned()]);
    }
}
