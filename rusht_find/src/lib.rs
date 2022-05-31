use ::std::io::{stdin, BufRead};
use ::std::process::exit;
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::sync::Arc;
use ::std::thread::{sleep, spawn};
use ::std::time::Duration;

use ::ustr::Ustr;

pub use self::unique::unique;
pub use self::unique::unique_prefix;
pub use self::unique::UniqueArgs;

mod unique;

pub fn read_input_lines() -> Vec<Ustr> {
    let has_data = Arc::new(AtomicBool::new(false));
    let has_data_setter = has_data.clone();
    start_time_monitor(has_data);
    perform_read_input_lines(has_data_setter)
}

fn perform_read_input_lines(has_data: Arc<AtomicBool>) -> Vec<Ustr> {
    stdin()
        .lock()
        .lines()
        .map(|line| Ustr::from(&line.expect("a line was not utf8")))
        .inspect(|_| has_data.store(true, Ordering::Release))
        .collect::<Vec<Ustr>>()
}

fn start_time_monitor(has_data: Arc<AtomicBool>) {
    spawn(move || {
        sleep(Duration::from_secs(3));
        if !has_data.load(Ordering::Acquire) {
            eprintln!("no input on stdin so far")
        }
        sleep(Duration::from_secs(30));
        if !has_data.load(Ordering::Acquire) {
            eprintln!("no input on stdin, terminating")
        }
        exit(1);
    });
}
