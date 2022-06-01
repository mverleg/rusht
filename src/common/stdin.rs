use ::std::env;
use ::std::io::{BufRead, stdin};
use ::std::process::exit;
use ::std::sync::Arc;
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::thread::{sleep, spawn};
use ::std::time::Duration;

use ::log::debug;
use ::log::error;
use ::log::trace;
use ::log::warn;

#[derive(Debug)]
pub enum EmptyLineHandling {
    Keep,
    Drop,
}

pub fn stdin_lines(empty: EmptyLineHandling) -> Vec<String> {
    debug!("reading lines from stdin");
    let has_data = Arc::new(AtomicBool::new(false));
    let has_data_setter = has_data.clone();
    start_time_monitor(has_data);
    perform_read_input_lines(has_data_setter, empty)
}

fn perform_read_input_lines(has_data: Arc<AtomicBool>, empty: EmptyLineHandling) -> Vec<String> {
    stdin()
        .lock()
        .lines()
        .map(|line| line.expect("failed to read line from stdin; not utf8?"))
        .inspect(|line| trace!("stdin line: {}", line))
        .inspect(|_| has_data.store(true, Ordering::Release))
        .filter(|line| matches!(empty, EmptyLineHandling::Keep) || !line.trim().is_empty())
        .collect::<Vec<_>>()
}

fn start_time_monitor(has_data: Arc<AtomicBool>) {
    spawn(move || {
        let timeout = read_timeout_env();
        sleep(Duration::from_millis(timeout * 100));
        if !has_data.load(Ordering::Acquire) {
            eprintln!("no input on stdin so far")
        }
        sleep(Duration::from_millis(timeout * 900));
        if !has_data.load(Ordering::Acquire) {
            eprintln!("no input on stdin, terminating (set STDIN_READ_TIMEOUT to extend)")
        }
        error!("timeout {} s, terminating", timeout);
        exit(1);
    });
}

fn read_timeout_env() -> u64 {
    match env::var("STDIN_READ_TIMEOUT") {
        Ok(timeout_str) => {
            match timeout_str.parse::<u64>() {
                Ok(timeout) => {
                    debug!("read STDIN_READ_TIMEOUT = {} seconds", timeout);
                    timeout
                }
                Err(_) => {
                    warn!("failed to parse STDIN_READ_TIMEOUT = {}, not a positive number, using default", timeout_str);
                    30
                }
            }
        }
        Err(_) => {
            debug!("did not find STDIN_READ_TIMEOUT in env, using default");
            30
        }
    }
}
