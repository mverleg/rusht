use ::std::cmp::max;
use ::std::cmp::min;
use ::std::io;

use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::sha2::Digest;
use ::sha2::Sha256;
use log::debug;

use super::NamesafeArgs;

pub fn namesafe(
    mut args: NamesafeArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut out_line_handler: impl FnMut(&str)
) -> Result<(), String> {
    if args.max_length < 8 {
        debug!("maximum length too low ({}), setting to 8", args.max_length);
        args.max_length = 8
    }
    while let Some(line_res) = line_supplier() {
        let oldline = line_res.map_err(|err| format!("failed to read line, {}", err))?;
        let newline = namesafe_line(&oldline, &args);
        out_line_handler(&newline)
    }
    Ok(())
}

pub fn namesafe_line(original: &str, args: &NamesafeArgs) -> String {
    debug_assert!(args.max_length >= 8);
    assert!(!args.keep_extension, "keeping extension not yet supported");  //TODO @mverleg:
    let mut count = 0;
    let max_length = max(8, args.max_length as usize);
    let mut is_prev_special = true;
    let mut filtered = original.chars()
        .map(|c| if args.charset.is_allowed(c) { c } else { '_' })
        .filter(|c| skip_subsequent_special(*c, &mut is_prev_special))
        .inspect(|_| count += 1)
        .take((max_length + 1) as usize)
        .collect::<String>();
    dbg!(&filtered);
    let was_changed = original == filtered;
    let was_too_long = count > max_length;
    let do_hash = args.hash_policy.should_hash(was_changed, was_too_long);
    if ! do_hash {
        return filtered;
    }
    if ! is_prev_special {
        filtered.push('_')
    }
    let hash_length = min(12, max_length / 2);
    let hash = compute_hash(&original, hash_length);
    let text_len = args.max_length as usize - hash.len();
    // use iterator because string slice can break up characters
    let mut new = filtered.chars()
        .take(text_len)
        .collect::<String>();
    new.push_str(&hash);
    new
}

fn skip_subsequent_special(symbol: char, is_prev_special: &mut bool) -> bool {
    if ! *is_prev_special {
        return true
    }
    let is_special = symbol == '_' || symbol == '-';
    if is_special {
        return false
    }
    *is_prev_special = is_special;
    return true;
}

fn compute_hash(text: &str, hash_length: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    let encoded = encode_config(hash_out, URL_SAFE_NO_PAD);
    encoded[..hash_length].to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use crate::escape::HashPolicy;
    use super::*;

    #[test]
    fn short_legal_filename() {
        let res = namesafe_line("Hello world", &NamesafeArgs::default());
        assert_eq!(res, "Hello_world");
    }

    #[test]
    fn short_legal_filename_hash() {
        let res = namesafe_line("Hello world", &NamesafeArgs {
            hash_policy: HashPolicy::Always,
            ..Default::default()
        });
        assert_eq!(res, "Hello_world_zoyiygcyaow6");
    }

    #[test]
    fn long_illegal_filename() {
        let res = namesafe_line(
            " _ hello WORLD hello world 你好 你好 你好 hello world- !!! !@#$%^& bye 123",
            &NamesafeArgs::default(),
        );
        assert_eq!(res, "hello_WORLD_hello_wozc4zyofxrnr1");
    }
}
