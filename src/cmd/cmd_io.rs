use ::std::fs::create_dir_all;
use ::std::fs::remove_file;
use ::std::fs::File;
use ::std::fs::OpenOptions;
use ::std::io::BufReader;
use ::std::io::BufWriter;
use ::std::io::Write;

use ::std::path::Path;
use ::std::path::PathBuf;

use ::std::time::SystemTime;
use ::std::time::UNIX_EPOCH;

use ::log::debug;

use ::memoize::memoize;
use ::regex::Regex;

use crate::cmd::cmd_type::DATA_VERSION;

use crate::cmd::cmd_type::TaskStack;
use crate::common::fail;

pub fn read(namespace: String) -> TaskStack {
    debug!("going to read commands for namespace '{}'", &namespace);
    let pth = stack_pth(namespace);
    if !pth.exists() {
        debug!("no commands file at '{}'", pth.to_string_lossy());
        return TaskStack::empty();
    }
    let reader = BufReader::new(open_file(&pth, false));
    match serde_json::from_reader::<_, TaskStack>(reader) {
        Ok(tasks) => {
            debug!(
                "successfully read {} commands from '{}'",
                tasks.len(),
                pth.to_string_lossy()
            );
            tasks
        }
        Err(err) => fail(&format!(
            "failed to parse commands in '{}', error: {}",
            pth.to_string_lossy(),
            err
        )),
    }
}

pub fn write(namespace: String, tasks: &TaskStack) {
    debug!("going to write commands for namespace '{}'", &namespace);
    let pth = stack_pth(namespace);
    if tasks.is_empty() {
        if pth.exists() {
            debug!(
                "commands stack is empty, deleting commands file at '{}'",
                pth.to_string_lossy()
            );
            if let Err(err) = remove_file(&pth) {
                fail(&format!(
                    "failed to remove commands in '{}', error: {}",
                    pth.to_string_lossy(),
                    err
                ));
            }
        } else {
            debug!(
                "commands stack is empty, there is no commands file at '{}', doing nothing",
                pth.to_string_lossy()
            );
        }
    } else {
        let mut writer = BufWriter::new(open_file(&pth, true));
        if let Err(err) = serde_json::to_writer_pretty(&mut writer, tasks) {
            fail(&format!(
                "failed to write commands in {}, error: {}",
                pth.to_string_lossy(),
                err
            ));
        }
        assert_eq!(writer.write(&[b'\n']).unwrap(), 2);
        debug!(
            "wrote updated commands file with {} commands to '{}'",
            tasks.len(),
            pth.to_string_lossy()
        );
    }
}

#[memoize]
pub fn stack_pth(namespace: String) -> PathBuf {
    let mut pth = make_app_dir();
    let filename = make_filename(namespace.clone());
    debug!("commands file for namespace '{}' and version {} is called '{}' inside cache directory '{}'",
            &namespace, DATA_VERSION, &filename, pth.to_string_lossy());
    pth.push(&filename);
    pth
}

fn make_app_dir() -> PathBuf {
    let mut pth = match dirs::cache_dir() {
        Some(pth) => pth,
        None => fail("failed to filter app data directory"),
    };
    pth.push("cmdstack");
    if let Err(err) = create_dir_all(&pth) {
        fail(format!(
            "failed to create directory {}, error {}",
            pth.to_string_lossy(),
            err
        ))
    }
    pth
}

fn make_filename(namespace: String) -> String {
    if namespace.is_empty() {
        return format!("cmd_stack_v{}.json", DATA_VERSION);
    }
    let re = Regex::new("^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9_-]*[a-zA-Z0-9])$").unwrap();
    if !re.is_match(&namespace) {
        fail("namespace should only contains alphanumeric characters, dashes and underscores, starting and ending with alphanumeric");
    }
    format!(
        "cmd_stack_{}_v{}.json",
        namespace.to_lowercase(),
        DATA_VERSION
    )
}

fn open_file(pth: &Path, write: bool) -> File {
    let mut opts = OpenOptions::new();
    if write {
        opts.write(true).truncate(true).create(true)
    } else {
        opts.read(true)
    };
    match opts.open(pth) {
        Ok(file) => file,
        Err(err) => {
            fail(&format!(
                "failed to open commands file at '{}' with options {:?}, error {}",
                pth.to_string_lossy(),
                &opts,
                err
            ));
        }
    }
}

pub fn current_time_s() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stack_pth() {
        assert_eq!(
            stack_pth("".to_owned()).file_name().unwrap(),
            format!("cmd_stack_v{}.json", DATA_VERSION).as_str()
        );
    }

    #[test]
    fn namespaced_stack_pth() {
        assert_eq!(
            stack_pth("1".to_owned()).file_name().unwrap(),
            format!("cmd_stack_1_v{}.json", DATA_VERSION).as_str()
        );
    }
}
