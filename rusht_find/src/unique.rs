use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "unique_prefix", about = "Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested.")]
pub struct UniqueArgs {
    #[structopt(parse(from_flag = Order::from_is_sorted), short = "s", long = "sorted", help = "Sort the entries")]
    pub order: Order,
    #[structopt(parse(from_flag = Keep::from_find_duplicates), short = "d", long = "find-duplicates", help = "Invert the behaviour, returning all first occurrences and keeping any subsequent duplicates.", conflicts_with="prefix", )]
    pub keep: Keep,
    #[structopt(short = "p", long = "prefix", help = "Remove any lines for which any other line is a prefix (including duplicates). E.g. /a and /a/b will remove the latter.")]
    pub prefix: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Order {
    #[default]
    Preserve,
    SortAscending,
}

impl Order {
    fn from_is_sorted(is_sorted: bool) -> Self {
        if is_sorted {
            Order::SortAscending
        } else {
            Order::Preserve
        }
    }

    fn order_inplace<T: Ord>(&self, data: &mut Vec<T>) {
        if let Order::SortAscending = *self {
            debug!("sorting unique_prefix result");
            data.sort_unstable()
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Keep {
    #[default]
    First,
    Subsequent,
}

impl Keep {
    fn from_find_duplicates(is_find_duplicates: bool) -> Self {
        if is_find_duplicates {
            Keep::Subsequent
        } else {
            Keep::First
        }
    }

    fn keep_is_first(&self, is_first: bool) -> bool {
        match self {
            Keep::First => !is_first,
            Keep::Subsequent => is_first,
        }
    }
}

pub fn unique<S>(texts: &[S], order: Order, keep: Keep) -> Vec<String>
        where S: AsRef<str>, S: Into<String> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
        if keep.keep_is_first(seen.insert(txt)) {
            continue
        }
        result.push(txt.into())
    }
    order.order_inplace(&mut result);
    result
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
pub fn unique_prefix<S>(texts: &[S], order: Order) -> Vec<String>
        where S: AsRef<str>, S: Into<String> {
    let mut known = unique(texts, Order::SortAscending, Keep::First);
    debug!("finding unique_prefix in {} items ({} unique)", texts.len(), known.len());
    let input = unique(texts, order, Keep::First);
    let mut result = Vec::with_capacity(known.len());
    for txt in input {
        let indx = known.binary_search(&txt)
            .expect("should always be found since collections have the same elements");
        if indx > 0 {
            let other = &known[indx - 1];
            eprintln!("-1: compare {} to {}", txt, other);  //TODO @mark:
            if txt.starts_with(other) {
                eprintln!("  DROP {}", txt);  //TODO @mark:
                known[indx - 1] = txt;
                continue
            }
        }
        result.push(txt.into())
    }
    order.order_inplace(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_first() {
        let res = unique(&vec!["/a", "/c", "/a", "/b"], Order::Preserve, Keep::First);
        assert_eq!(res, vec!["/a".to_owned(), "/c".to_owned(), "/b".to_owned()]);
    }

    #[test]
    fn unique_sorted() {
        let res = unique(&vec!["/a", "/c", "/a", "/b"], Order::SortAscending, Keep::First);
        assert_eq!(res, vec!["/a".to_owned(), "/b".to_owned(), "/c".to_owned()]);
    }

    #[test]
    fn unique_duplicates() {
        let res = unique(&vec!["/a", "/c", "/a", "/a", "/b", "/c"], Order::Preserve, Keep::Subsequent);
        assert_eq!(res, vec!["/a".to_owned(), "/a".to_owned(), "/c".to_owned()]);
    }

    #[test]
    fn unique_prefix_first() {
        let res = unique_prefix(&vec!["/a", "/a/b", "/a/c"], Order::Preserve);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn unique_prefix_duplicates() {
        let res = unique_prefix(&vec!["/a", "/a", "/a"], Order::Preserve);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn unique_prefix_middle() {
        let res = unique_prefix(&vec!["/a/c", "/a", "/a/b"], Order::Preserve);
        assert_eq!(res, vec!["/a".to_owned()]);
    }

    #[test]
    fn unique_prefix_sorted() {
        let res = unique_prefix(&vec!["/a/c", "/a/b", "/a/c/q"], Order::SortAscending);
        assert_eq!(res, vec!["/a/b".to_owned(), "/a/b".to_owned()]);
    }

    #[test]
    fn unique_prefix_nomatch() {
        let res = unique_prefix(&vec!["/a/c", "/a/b", "/b"], Order::Preserve);
        assert_eq!(res, vec!["/a/c".to_owned(), "/a/b".to_owned(), "/b".to_owned()]);
    }
}
