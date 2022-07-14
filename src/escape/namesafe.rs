use ::std::cmp::min;
use ::std::io;
use base64::{encode_config, URL_SAFE_NO_PAD};
use sha2::Sha256;

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

pub fn namesafe_line(original: &str, args: &NamesafeArgs) -> String {
    let mut count = 0;
    //TODO @mverleg: remove subsequent dashses/underscores
    let filtered = original.chars()
        .filter(|c| args.charset.is_allowed(*c))
        .inspect(|_| count += 1)
        .take((args.max_length + 1) as usize)
        .collect::<String>();
    let was_changed = original == filtered;
    let was_too_long = count > args.max_length;
    let do_hash = args.hash_policy.should_hash(was_changed, was_too_long);
    if ! do_hash {
        return filtered;
    }
    let hash_length = min(12, args.max_length / 2);
    let hash = compute_hash(&original, hash_length);
    let text_len = args.max_length - hash.len();
    format!("{}{}", filtered[..text_len], hash)
}

fn compute_hash(text: &str, hash_length: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    encode_config(hash_out, URL_SAFE_NO_PAD)[..20].to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_legal_filename() {
        let res = namesafe_line("Hello world", &NamesafeArgs::default());
        assert_eq!(res, "Hello_world_zoyiygcyaow6gjvnihtt");
    }

    #[test]
    fn long_illegal_filename() {
        let res = namesafe_line(
            " _ hello WORLD hello world 你好 你好 你好 hello world- !!! !@#$%^& bye 123",
            &NamesafeArgs::default(),
        );
        assert_eq!(res, "hello_WORLD_hello_world_hello_wo_zc4zyofxrnr1onvipg5w");
    }
}
