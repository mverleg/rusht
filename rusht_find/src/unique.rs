use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;
use ::ustr::Ustr;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "unique_prefix", about = "Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested.")]
pub struct UniqueArgs {
    #[structopt(parse(from_flag = Order::from_is_sorted), short = "s", long = "sorted", help = "Sort the entries")]
    pub order: Order,
    #[structopt(parse(from_flag = Keep::from_find_duplicates), short = "d", long = "find-duplicates", help = "Invert the behaviour, returning all first occurrences and keeping any subsequent duplicates.", conflicts_with = "prefix", )]
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

pub fn unique(texts: Vec<Ustr>, order: Order, keep: Keep) -> Vec<Ustr> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        if keep.keep_is_first(seen.insert(txt)) {
            continue;
        }
        result.push(txt.into())
    }
    order.order_inplace(&mut result);
    result
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
pub fn unique_prefix(mut texts: Vec<Ustr>, order: Order) -> Vec<Ustr> {
    texts.sort_unstable();
    let mut result = Vec::with_capacity(texts.len());
    let mut prev = texts[0].as_str();
    for indx in 1 .. texts.len() {
        let prev_is_parent = texts[indx].as_str().starts_with(prev);
        if prev_is_parent {
            eprintln!("skipping {} because of {}", texts[indx], prev);  //TODO @mark: TEMPORARY! REMOVE THIS!
            continue
        }
        eprintln!("including {} despite {}", texts[indx], prev);  //TODO @mark: TEMPORARY! REMOVE THIS!
        prev = texts[indx].as_str();
        result.push(texts[indx].into())
    }
    order.order_inplace(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use super::*;

    macro_rules! ustrvec {
        ($($element: expr),*) => {
            {
                let mut txts: Vec<Ustr> = Vec::new();
                $(
                    txts.push(Ustr::from(&$element));
                )*
                txts
            }
        };
    }

    #[test]
    fn unique_first() {
        let res = unique(ustrvec!["/a", "/c", "/a", "/b"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a", "/c", "/b"]);
    }

    #[test]
    fn unique_sorted() {
        let res = unique(ustrvec!["/a", "/c", "/a", "/b"], Order::SortAscending, Keep::First);
        assert_eq!(res, ustrvec!["/a", "/b", "/c"]);
    }

    #[test]
    fn unique_duplicates() {
        let res = unique(ustrvec!["/a", "/c", "/a", "/a", "/b", "/c"], Order::Preserve, Keep::Subsequent);
        assert_eq!(res, ustrvec!["/a", "/a", "/c"]);
    }

    #[test]
    fn unique_prefix_first() {
        let res = unique_prefix(ustrvec!["/a", "/a/b", "/a/c", "/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_duplicates() {
        let res = unique_prefix(ustrvec!["/a", "/a", "/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_middle() {
        let res = unique_prefix(ustrvec!["/a/c", "/a", "/a/b"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_sorted() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/b", "/a/c/q"], Order::SortAscending);
        assert_eq!(res, ustrvec!["/a/b", "/a/b"]);
    }

    #[test]
    fn unique_prefix_nomatch() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/b", "/b"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a/c", "/a/b", "/b"]);
    }

    #[test]
    fn unique_prefix_dedup_if_no_parent() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/c", "/b", "/b/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a/c", "/b"]);
    }

    #[ignore]
    #[test]
    fn ustr_order_operations() {
        // problem with ustr
        assert_eq!(Ustr::from("/a/b").partial_cmp(&Ustr::from("/a/c")).unwrap(), Ordering::Less);
        assert_eq!(Ustr::from("/a/b").cmp(&Ustr::from("/a/c")), Ordering::Less);
        assert_eq!(Ustr::from("/a/c").partial_cmp(&Ustr::from("/a/b")).unwrap(), Ordering::Greater);
        assert_eq!(Ustr::from("/a/c").cmp(&Ustr::from("/a/b")), Ordering::Greater);
    }
}
