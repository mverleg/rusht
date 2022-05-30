use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;
use ::ustr::Ustr;
use ::ustr::UstrMap;

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

pub fn unique(texts: &[Ustr], order: Order, keep: Keep) -> Vec<Ustr> {
    let mut result = Vec::with_capacity(texts.len());
    let mut seen = HashSet::with_capacity(texts.len());
    for txt in texts {
        let txt = txt.as_ref();
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
pub fn unique_prefix(texts: &[Ustr], order: Order) -> Vec<Ustr> {
    let mut known = unique(texts, Order::SortAscending, Keep::First);
    debug!("finding unique_prefix in {} items ({} unique)", texts.len(), known.len());
    let mut result = Vec::with_capacity(known.len());
    let mut parents = UstrMap::default();
    for txt in texts {
        if parents.contains_key(txt) {
            eprintln!("  duplicate {}", txt);
            continue;
        }
        dbg!(&known);  //TODO @mark: TEMPORARY! REMOVE THIS!
        dbg!(&txt);  //TODO @mark: TEMPORARY! REMOVE THIS!
        dbg!(known.binary_search(&txt));  //TODO @mark: TEMPORARY! REMOVE THIS!
        let indx = known.binary_search(&txt)
            .expect("should always be found since collections have the same elements");
        // dbg!(indx);  //TODO @mark: TEMPORARY! REMOVE THIS!
        if indx > 0 {
            let other = &known[indx - 1];
            eprintln!("-1: compare {} to {}", txt, other);  //TODO @mark:
            if txt.as_str().starts_with(other.as_str()) {
                parents.insert(txt.clone(), other);
                // if let Some(other_parent) = parents.get(other) {
                //
                // } else {
                //
                // }
                eprintln!("  DROP {}", txt);  //TODO @mark:
                //known[indx - 1] = txt;
                continue;
            }
        }
        result.push(txt.clone())
    }
    order.order_inplace(&mut result);
    result
}

#[cfg(test)]
mod tests {
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
        let res = unique(&ustrvec!["/a", "/c", "/a", "/b"], Order::Preserve, Keep::First);
        assert_eq!(res, ustrvec!["/a", "/c", "/b"]);
    }

    #[test]
    fn unique_sorted() {
        let res = unique(&ustrvec!["/a", "/c", "/a", "/b"], Order::SortAscending, Keep::First);
        assert_eq!(res, ustrvec!["/a", "/b", "/c"]);
    }

    #[test]
    fn unique_duplicates() {
        let res = unique(&ustrvec!["/a", "/c", "/a", "/a", "/b", "/c"], Order::Preserve, Keep::Subsequent);
        assert_eq!(res, ustrvec!["/a", "/a", "/c"]);
    }

    #[test]
    fn unique_prefix_first() {
        let res = unique_prefix(&ustrvec!["/a", "/a/b", "/a/c", "/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_duplicates() {
        let res = unique_prefix(&ustrvec!["/a", "/a", "/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_middle() {
        let res = unique_prefix(&ustrvec!["/a/c", "/a", "/a/b"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a"]);
    }

    #[test]
    fn unique_prefix_sorted() {
        let res = unique_prefix(&ustrvec!["/a/c", "/a/b", "/a/c/q"], Order::SortAscending);
        assert_eq!(res, ustrvec!["/a/b", "/a/b"]);
    }

    #[test]
    fn unique_prefix_nomatch() {
        let res = unique_prefix(&ustrvec!["/a/c", "/a/b", "/b"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a/c", "/a/b", "/b"]);
    }

    #[test]
    fn unique_prefix_dedup_if_no_parent() {
        let res = unique_prefix(&ustrvec!["/a/c", "/a/c", "/b", "/b/a"], Order::Preserve);
        assert_eq!(res, ustrvec!["/a/c", "/b"]);
    }

    #[test]
    fn check_for_a_binary_search_problem_that_happened_pure_string() {
        //TODO @mark: TEMPORARY! REMOVE THIS!
        let mut values = Vec::new();
        values.push("/a".to_owned());
        values.push("/a/b".to_owned());
        values.push("/a/c".to_owned());
        let mut sorted = values.clone();
        sorted.sort();
        assert_eq!(values, sorted);
        dbg!(&values);
        let find = values.binary_search(&"/a/c".to_owned());
        assert_eq!(find, Ok(2));
    }

    #[test]
    fn ustr_order_operations() {
        assert!(!(Ustr::from("/a/c") > Ustr::from("/a/c")));
        assert!(!(Ustr::from("/a/c") < Ustr::from("/a/c")));
        assert!(Ustr::from("/a/c") == Ustr::from("/a/c"));
        assert!(Ustr::from("/a/b") < Ustr::from("/a/c"));
    }

    #[test]
    fn check_for_a_binary_search_problem_that_happened() {
        let mut values = Vec::new();
        values.push(Ustr::from(&"/a"));
        values.push(Ustr::from(&"/a/b"));
        values.push(Ustr::from(&"/a/c"));
        let mut sorted = values.clone();
        sorted.sort();
        assert_eq!(values, sorted);
        dbg!(&values);
        let needle = Ustr::from("/a/c");
        let find = values.binary_search_by(|l| {
            eprintln!("{} vs {}: {:?}", l, needle, l.cmp(&needle));
            l.cmp(&needle)
        });
        assert_eq!(find, Ok(2));
    }
}
