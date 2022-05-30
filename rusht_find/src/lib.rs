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
        match known.binary_search(&txt) {
            Ok(indx) => { /* skip duplicate */ },
            Err(_) => {}
        }
    }
    result
}
