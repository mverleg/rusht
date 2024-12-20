use ::std::cmp::max;
use ::std::cmp::min;

use ::base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ::base64::Engine;
use ::log::debug;
use ::sha2::Digest;
use ::sha2::Sha256;

use crate::common::{LineReader, LineWriter};

use super::NamesafeArgs;

pub async fn namesafe(
    mut args: NamesafeArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> Result<(), String> {
    if args.max_length < 8 {
        debug!("maximum length too low ({}), setting to 8", args.max_length);
        args.max_length = 8
    }
    let mut any_line = false;
    while let Some(oldline) = reader.read_line().await {
        let mut newline = namesafe_line(&oldline, &args);
        if args.single_line && any_line {
            return Err("namesafe failed because it received more than one line, and --single was requested".to_owned());
        };
        if args.lowercase {
            newline = newline.to_lowercase();
        }
        writer.write_line(&newline).await;
        any_line = true;
    }
    if args.allow_empty || any_line {
        Ok(())
    } else {
        Err("namesafe failed because it did not receive any lines (use --allow-empty if this is okay)".to_owned())
    }
}

//TODO @mverleg: only pass relevant line args
pub fn namesafe_line(original: &str, args: &NamesafeArgs) -> String {
    debug_assert!(args.max_length >= 8);
    assert!(!args.keep_extension, "keeping extension not yet supported"); //TODO @mverleg:
    let mut count = 0;
    let max_length = max(8, args.max_length as usize);
    let mut is_prev_special = if args.allow_outer_connector {
        false
    } else {
        true
    };
    let separator_arg = if let Some(sep) = args.separator { sep } else { '_' };
    let mut filtered = original
        .chars()
        .map(|c| if args.charset.is_allowed(c, args.separator) { c } else { separator_arg })
        .filter(|c| skip_subsequent_special(*c, &mut is_prev_special, args.separator))
        .inspect(|_| count += 1)
        .collect::<String>();
    if !args.allow_outer_connector {
        while filtered.ends_with('_') || filtered.ends_with('-') || filtered.ends_with(separator_arg) {
            filtered.pop();
        }
    }
    let was_changed = original != filtered;
    let was_too_long = count > max_length;
    let do_hash = filtered.is_empty() || args.hash_policy.should_hash(was_changed, was_too_long);
    debug!(
        "for line {original}: was_changed={was_changed}, was_too_long={was_too_long}, \
            count={count}, max_length={max_length}, do_hash={do_hash}, hash_policy={0:?}",
        args.hash_policy
    );
    if !do_hash {
        return shorten(&filtered, count, max_length, args.keep_tail);
    }
    if !filtered.is_empty() {
        //TODO @mark: this might cause issues if allow_outer_connector and ends with _-
        filtered.push(separator_arg);
    }
    let hash_length = min(12, max_length / 2);
    let hash = compute_hash(original, hash_length);
    let text_len = args.max_length as usize - hash.len();
    let mut shortened = shorten(&filtered, count, text_len, args.keep_tail);
    shortened.push_str(&hash);
    shortened
}

fn shorten(filtered: &str, actual_len: usize, goal_len: usize, keep_tail: bool) -> String {
    // use iterator because string slice can break up characters
    if keep_tail {
        filtered
            .chars()
            .skip(actual_len.saturating_sub(goal_len))
            .collect::<String>()
    } else {
        filtered.chars().take(goal_len).collect::<String>()
    }
}

fn skip_subsequent_special(symbol: char, is_prev_special: &mut bool, separator_arg: Option<char>) -> bool {
    let is_special = match separator_arg {
        None => symbol == '_' || symbol == '-',
        Some(separator) => symbol == separator,
    };
    if is_special && *is_prev_special {
        return false;
    }
    *is_prev_special = is_special;
    true
}

fn compute_hash(text: &str, hash_length: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash_out = hasher.finalize();
    let encoded = URL_SAFE_NO_PAD.encode(hash_out);
    encoded[..hash_length].to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use ::clap::Parser;

    use crate::escape::HashPolicy;

    use super::*;

    #[test]
    fn already_valid() {
        let res = namesafe_line("Hello_world", &NamesafeArgs::default());
        assert_eq!(res, "Hello_world");
    }

    #[test]
    fn short_name_bug() {
        let res = namesafe_line(
            "vp",
            &NamesafeArgs {
                hash_policy: HashPolicy::TooLong,
                keep_tail: true,
                single_line: true,
                ..Default::default()
            },
        );
        assert_eq!(res, "vp");
    }

    #[test]
    fn legal_filename_hash() {
        let res = namesafe_line(
            "Hello world",
            &NamesafeArgs {
                hash_policy: HashPolicy::Always,
                ..Default::default()
            },
        );
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

    #[test]
    fn subsequent_weird_symbols() {
        let res = namesafe_line(
            "_-_hello!@#$%^&world-_-make-this-name-really-really-long",
            &NamesafeArgs {
                hash_policy: HashPolicy::Never,
                ..Default::default()
            },
        );
        assert_eq!(res, "hello_world-make-this-name-reall");
    }

    #[test]
    fn long_legal_filename_tail() {
        let res = namesafe_line(
            "The King is dead. Long live the Queen!",
            &NamesafeArgs {
                max_length: 15,
                keep_tail: true,
                hash_policy: HashPolicy::Never,
                ..Default::default()
            },
        );
        assert_eq!(res, "live_the_Queen");
    }

    #[test]
    fn dashes_and_underscores() {
        let res = namesafe_line(
            " _-_ ",
            &NamesafeArgs {
                hash_policy: HashPolicy::Never,
                ..Default::default()
            },
        );
        // use hash if result is too short
        assert_eq!(res, "cavx4zqano9q");
    }

    #[test]
    fn long_once_without_hash() {
        // based on real use bug
        let args = NamesafeArgs::parse_from(vec!["namesafe", "-1", "-x=n"]);
        let res = namesafe_line("commits-for-review-unpushed-firstN", &args);
        assert_eq!(res, "commits-for-review-unpushed-firs");
    }

    #[test]
    fn custom_separator() {
        let res = namesafe_line(
            "The King is dead. Long live the 皇帝! %^",
            &NamesafeArgs::parse_from(vec!["namesafe", "-x=l", "-E", "-S=/", "-u", "-l=24"]),
        );
        assert_eq!(res, "live/the/皇帝/844x4njhe2kx");
    }
}
