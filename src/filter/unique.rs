use ::std::collections::HashSet;
use ::std::io;

use ::clap::StructOpt;
use ::log::debug;
use ::regex::Regex;

use crate::common::get_matches;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "unique",
    about = "Remove any duplicate lines, keeping the first match and preserving order unless sorting is requested."
)]
pub struct UniqueArgs {
    #[structopt(parse(from_flag = Order::from_is_sorted), short = 's', long = "sorted", help = "Sort the entries. Buffers all the input.")]
    pub order: Order,
    #[structopt(parse(from_flag = Keep::from_find_duplicates), short = 'd', long = "filter-duplicates", help = "Invert the behaviour, returning all first occurrences and keeping any subsequent duplicates.", conflicts_with = "prefix", )]
    pub keep: Keep,
    #[structopt(
        long,
        help = "Use a given regular expression that captures the key to deduplicate by. Uses the first capture group if any, or the whole match otherwise. Only buffers per-line, i.e. near-real-time."
    )]
    pub by: Option<Regex>,
    #[structopt(
        short = 'p',
        long = "prefix",
        help = "Remove any lines for which any other line is a prefix (including duplicates). E.g. /a and /a/b will remove the latter. Buffers all the input.",
        conflicts_with = "by"
    )]
    pub prefix: bool,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    UniqueArgs::into_app().debug_assert()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
}

fn order_inplace<T: Ord>(data: &mut [T]) {
    debug!("sorting unique_prefix result");
    data.sort_unstable()
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

pub fn unique(
    args: UniqueArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut out_line_handler: impl FnMut(&str)
) {
    if args.prefix {
        let mut lines = vec![];
        while let Some(line_res) = line_supplier() {
            lines.push(line_res.expect("could not read line"))
        }
        unique_prefix(lines, args.order, args.keep).iter()
            .for_each(|line| out_line_handler(line.as_str()));
    } else {
        if Order::SortAscending == args.order {
            let mut matches = vec![];
            unique_nosort(args.keep, &args.by, line_supplier, |line| matches.push(line.to_owned()));
            order_inplace(&mut matches);
            matches.into_iter().for_each(|line| out_line_handler(&line))
        } else {
            unique_nosort(args.keep, &args.by, line_supplier, |line| out_line_handler(line))
        }
    };
}

fn unique_nosort(
    keep: Keep,
    unique_by_pattern: &Option<Regex>,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut out_line_handler: impl FnMut(&str),
) {
    let mut seen = HashSet::new();
    while let Some(line_res) = line_supplier() {
        let line = match line_res {
            Ok(line) => line,
            Err(err) => panic!("failed to read line: {}", err),
        };
        //TODO @mverleg: can this use a borrow somehow?
        let mut key = line.clone();
        if let Some(re) = unique_by_pattern {
            get_matches(re, &line, &mut |mtch| key = mtch, true, true)
        }
        if !keep.keep_is_first(seen.insert(key)) {
            continue;
        }
        out_line_handler(&line)
    }
}

/// Removes strings that have another string as prefix, preserving order.
/// E.g. '/a/b' and '/a/c' and '/a', will keep '/a'
pub fn unique_prefix(texts: Vec<String>, order: Order, keep: Keep) -> Vec<String> {
    if matches!(order, Order::SortAscending) && matches!(keep, Keep::Subsequent) {
        panic!("--filter-duplicates, --sorted and --prefix cannot all be used together");
    };
    if texts.is_empty() {
        debug!("empty input while removing items that have other items as prefix");
        return texts;
    }
    match order {
        Order::Preserve => {
            debug!("removing items that have other items as prefix, preserving order");
            let mut uniques = HashSet::with_capacity(texts.len());
            unique_prefix_sorted(texts.clone(), |uniq| {
                uniques.insert(uniq);
            });
            let mut seen: HashSet<String> = HashSet::default();
            texts
                .into_iter()
                .filter(|item| uniques.contains(item))
                .filter(|item| keep.keep_is_first(seen.insert(item.clone())))
                .collect()
        }
        Order::SortAscending => {
            debug!("removing items that have other items as prefix, sorting ascendingly");
            let mut result = Vec::with_capacity(texts.len());
            unique_prefix_sorted(texts, |uniq| result.push(uniq));
            result
        }
    }
}

fn unique_prefix_sorted(mut texts: Vec<String>, mut collect: impl FnMut(String)) {
    //TODO @mverleg: too many clones here...
    texts.sort_unstable();
    collect(texts[0].to_owned());
    let mut prev = texts[0].to_owned();
    for this in texts.into_iter().skip(1) {
        let prev_is_parent = this.as_str().starts_with(&prev);
        if prev_is_parent {
            continue;
        }
        prev = this.to_owned();
        collect(this)
    }
}

#[cfg(test)]
#[allow(clippy::vec_init_then_push, unused_mut)]
mod tests {
    use super::*;

    macro_rules! strvec {
        ($($element: expr),*) => {
            {
                let mut txts: Vec<String> = Vec::new();
                $(
                    txts.push($element.to_owned());
                )*
                txts
            }
        };
    }

    fn unique_collect(lines: Vec<String>, order: Order, keep: Keep) -> Vec<String> {
        let args = UniqueArgs { order, keep, by: None, prefix: false };
        let mut res = vec![];
        let mut line_iter = lines.into_iter().map(|line| io::Result::Ok(line));
        unique(args, || line_iter.next(), |line| res.push(line.to_owned()));
        res
    }

    #[test]
    fn unique_first() {
        let res = unique_collect(
            strvec!["/a", "/c", "/a", "/b"],
            Order::Preserve,
            Keep::First,
        );
        assert_eq!(res, strvec!["/a", "/c", "/b"]);
    }

    #[test]
    fn unique_sorted() {
        let res = unique_collect(
            strvec!["/a", "/c", "/a", "/b"],
            Order::SortAscending,
            Keep::First,
        );
        assert_eq!(res, strvec!["/a", "/b", "/c"]);
    }

    #[test]
    fn unique_duplicates() {
        let res = unique_collect(
            strvec!["/a", "/c", "/a", "/a", "/b", "/c"],
            Order::Preserve,
            Keep::Subsequent,
        );
        assert_eq!(res, strvec!["/a", "/a", "/c"]);
    }

    //TODO @mverleg:
    // #[test]
    // fn unique_by() {
    //     let mut res = vec![];
    //     unique_live_preserve_order(
    //         strvec!["hello world", "hello moon", "bye moon"],
    //         Keep::First,
    //         &Some(Regex::new("^[a-z]+").unwrap()),
    //         |line| res.push(line.to_owned())
    //     );
    //     assert_eq!(res, vec!["hello world".to_owned(), "bye moon".to_owned()]);
    // }

    #[test]
    fn unique_prefix_empty() {
        let res = unique_prefix(strvec![], Order::Preserve, Keep::First);
        assert_eq!(res, strvec![]);
    }

    #[test]
    fn unique_prefix_first() {
        let res = unique_prefix(
            strvec!["/a", "/a/b", "/a/c", "/a"],
            Order::Preserve,
            Keep::First,
        );
        assert_eq!(res, strvec!["/a"]);
    }

    #[test]
    fn unique_prefix_drop_duplicates() {
        let res = unique_prefix(strvec!["/a", "/a", "/a"], Order::Preserve, Keep::First);
        assert_eq!(res, strvec!["/a"]);
    }

    #[test]
    fn unique_prefix_middle() {
        let res = unique_prefix(strvec!["/a/c", "/a", "/a/b"], Order::Preserve, Keep::First);
        assert_eq!(res, strvec!["/a"]);
    }

    #[test]
    fn unique_prefix_keep_duplicates() {
        let res = unique_prefix(
            strvec!["/a/c", "/a", "/a/b", "/a/c", "/a", "/a"],
            Order::Preserve,
            Keep::Subsequent,
        );
        assert_eq!(res, strvec!["/a", "/a"]);
    }

    #[test]
    #[should_panic]
    fn unique_prefix_keep_duplicates_not_supported_with_sort() {
        let _ = unique_prefix(strvec![], Order::SortAscending, Keep::Subsequent);
    }

    #[test]
    fn unique_prefix_preserve_order() {
        let res = unique_prefix(
            strvec!["/d", "/b", "/a", "/c", "/a/a"],
            Order::Preserve,
            Keep::First,
        );
        assert_eq!(res, strvec!["/d", "/b", "/a", "/c"]);
    }

    #[test]
    fn unique_prefix_sorted() {
        let res = unique_prefix(
            strvec!["/a/c", "/a/b", "/a/c/q"],
            Order::SortAscending,
            Keep::First,
        );
        assert_eq!(res, strvec!["/a/b", "/a/c"]);
    }

    #[test]
    fn unique_prefix_nomatch() {
        let res = unique_prefix(strvec!["/a/c", "/a/b", "/b"], Order::Preserve, Keep::First);
        assert_eq!(res, strvec!["/a/c", "/a/b", "/b"]);
    }

    #[test]
    fn unique_prefix_dedup_if_no_parent() {
        let res = unique_prefix(
            strvec!["/a/c", "/a/c", "/b", "/b/a"],
            Order::Preserve,
            Keep::First,
        );
        assert_eq!(res, strvec!["/a/c", "/b"]);
    }
}
