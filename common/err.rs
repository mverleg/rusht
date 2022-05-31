use ::std::process::exit;

use ::log::warn;

pub fn fail(msg: impl AsRef<str>) -> ! {
    let msg = msg.as_ref();
    warn!("{}", msg);
    eprintln!("{}", msg);
    debug_assert!(false, "explicit `fail` was called");
    exit(2)
}
