use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;
use ::ustr::Ustr;
use ::ustr::UstrSet;

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
            Keep::First => is_first,
            Keep::Subsequent => !is_first,
        }
    }
}

pub fn unique(texts: Vec<Ustr>, order: Order, keep: Keep) -> Vec<Ustr> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        if ! keep.keep_is_first(seen.insert(txt)) {
            continue;
        }
        result.push(txt.into())
    }
    order.order_inplace(&mut result);
    result
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
pub fn unique_prefix(texts: Vec<Ustr>, order: Order, keep: Keep) -> Vec<Ustr> {
    if matches!(order, Order::SortAscending) && matches!(keep, Keep::Subsequent) {
        panic!("--find-duplicates, --sorted and --prefix cannot all be used together");
    };
    if texts.is_empty() {
        debug!("empty input while removing items that have other items as prefix");
        return texts
    }
    match order {
        Order::Preserve => {
            debug!("removing items that have other items as prefix, preserving order");
            let mut uniques = HashSet::with_capacity(texts.len());
            unique_prefix_sorted(texts.clone(), |uniq| { uniques.insert(uniq); });
            let mut seen = UstrSet::default();
            texts.into_iter()
                .filter(|item| uniques.contains(item))
                .filter(|item| keep.keep_is_first(seen.insert(item.clone())))
                .collect()
        },
        Order::SortAscending => {
            debug!("removing items that have other items as prefix, sorting ascendingly");
            let mut result = Vec::with_capacity(texts.len());
            unique_prefix_sorted(texts, |uniq| result.push(uniq));
            result
        },
    }
}

fn unique_prefix_sorted(mut texts: Vec<Ustr>, mut collect: impl FnMut(Ustr)) {
    texts.sort_unstable();
    collect(texts[0].into());
    let mut prev = texts[0].as_str();
    for indx in 1 .. texts.len() {
        let this = texts[indx];
        let prev_is_parent = this.as_str().starts_with(prev);
        if prev_is_parent {
            eprintln!("{}: drop {} because of {}", indx, this, prev);  //TODO @mark:
            continue
        }
        eprintln!("{}: keep {} despite {}", indx, this, prev);  //TODO @mark:
        prev = this.as_str();
        collect(this.into())
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::panic::catch_unwind;

    use super::*;

    macro_rules! ustrvec {
        ($($element: expr),*) => {
            {
                #[allow(unused_mut)]
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
    fn unique_prefix_empty() {
        let res = unique_prefix(ustrvec![], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec![]);
    }

    #[test]
    fn unique_prefix_first() {
        let res = unique_prefix(ustrvec!["/a", "/a/b", "/a/c", "/a"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_drop_duplicates() {
        let res = unique_prefix(ustrvec!["/a", "/a", "/a"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_middle() {
        let res = unique_prefix(ustrvec!["/a/c", "/a", "/a/b"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_keep_duplicates() {
        let res = unique_prefix(ustrvec!["/a/c", "/a", "/a/b", "/a/c", "/a", "/a"], Order::Preserve, Keep::Subsequent);
        assert_eq!(res, ustrvec!["/a", "/a"]);
    }

    #[test]
    #[should_panic]
    fn unique_prefix_keep_duplicates_not_supported_with_sort() {
        let _ = unique_prefix(ustrvec![], Order::SortAscending, Keep::Subsequent);
    }

    #[test]
    fn unique_prefix_preserve_order() {
        let res = unique_prefix(ustrvec!["/d", "/b", "/a", "/c", "/a/a"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/d", "/b", "/a", "/c"]);
    }

    #[test]
    fn unique_prefix_sorted() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/b", "/a/c/q"], Order::SortAscending, Keep::First);
        assert_eq!(res, ustrvec!["/a/b", "/a/c"]);
    }

    #[test]
    fn unique_prefix_nomatch() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/b", "/b"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a/c", "/a/b", "/b"]);
    }

    #[test]
    fn unique_prefix_dedup_if_no_parent() {
        let res = unique_prefix(ustrvec!["/a/c", "/a/c", "/b", "/b/a"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a/c", "/b"]);
    }

    #[ignore]
    #[test]
    fn ustr_order_operations() {
        // problem with ustr, has a fix but not published; doesn't matter anymore since no longer using trees
        assert_eq!(Ustr::from("/a/b").partial_cmp(&Ustr::from("/a/c")).unwrap(), Ordering::Less);
        assert_eq!(Ustr::from("/a/b").cmp(&Ustr::from("/a/c")), Ordering::Less);
        assert_eq!(Ustr::from("/a/c").partial_cmp(&Ustr::from("/a/b")).unwrap(), Ordering::Greater);
        assert_eq!(Ustr::from("/a/c").cmp(&Ustr::from("/a/b")), Ordering::Greater);
    }
}
