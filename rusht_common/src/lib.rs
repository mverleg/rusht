use ::std::io::{BufRead, stdin};
use ::std::process::exit;
use ::std::sync::Arc;
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::thread::{sleep, spawn};
use ::std::time::Duration;

use ::log::debug;
use ::log::trace;

pub fn stdin_lines() -> Vec<String> {
    debug!("reading lines from stdin");
    let has_data = Arc::new(AtomicBool::new(false));
    let has_data_setter = has_data.clone();
    start_time_monitor(has_data);
    perform_read_input_lines(has_data_setter)
}

fn perform_read_input_lines(has_data: Arc<AtomicBool>) -> Vec<String> {
    stdin()
        .lock()
        .lines()
        .map(|line| line.expect("failed to read line from stdin; not utf8?"))
        .inspect(|line| trace!("stdin line: {}", line))
        .inspect(|_| has_data.store(true, Ordering::Release))
        .collect::<Vec<_>>()
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
