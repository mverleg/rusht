use ::std::cmp::min;
use ::std::io;

use super::NamesafeArgs;

pub fn namesafe(
    args: NamesafeArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut out_line_handler: impl FnMut(&str)
) -> Result<(), String> {
    while let Some(line_res) = line_supplier() {
        let oldline = line_res.map_err(|err| format!("failed to read line, {}", err))?;
        let newline = namesafe_line(&oldline, &args);
        out_line_handler(&newline)
    }
    Ok(())
}

fn namesafe_line(line: &str, args: &NamesafeArgs) -> String {
    let mut count = 0;
    let filtered = line.chars()
        .filter(|c| args.charset.is_allowed(*c))
        .inspect(|_| count += 1)
        .take((args.max_length + 1) as usize)
        .collect::<String>();
    let was_changed = line == filtered;
    let was_too_long = count > args.max_length;
    let do_hash = args.hash_policy.should_hash(was_changed, was_too_long);
    if ! do_hash {
        return filtered;
    }
    let hash_length = min(10, args.max_length / 2);
    filtered;
}

/// Turn a text into a safe filename, trying to keep it unique.
pub fn unique_filename(text: &str) -> String {
    //TODO @mverleg: remove
    let clean = INVALID_CHARS.replace_all(text, "_");
    let squash = SQUASH_CHARS.replace_all(clean.as_ref(), "_");
    let trim = INVALID_EDGES.replace_all(squash.as_ref(), "");
    let short: String = trim.as_ref().chars().take(32).collect();
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    let hash = encode_config(hash_out, URL_SAFE_NO_PAD)[..20].to_ascii_lowercase();
    format!("{short}_{hash}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_legal_filename() {
        let res = unique_filename("Hello world");
        assert_eq!(res, "Hello_world_zoyiygcyaow6gjvnihtt");
    }

    #[test]
    fn long_illegal_filename() {
        let res = unique_filename(
            " _ hello WORLD hello world 你好 你好 你好 hello world- !!! !@#$%^& bye 123",
        );
        assert_eq!(res, "hello_WORLD_hello_world_hello_wo_zc4zyofxrnr1onvipg5w");
    }
}
