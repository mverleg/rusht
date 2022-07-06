use ::std::fmt;
use ::std::fmt::Formatter;
use ::std::fs::create_dir_all;
use ::std::fs::File;
use ::std::fs::OpenOptions;
use ::std::fs::remove_file;
use ::std::io::BufReader;
use ::std::io::BufWriter;
use ::std::io::Write;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::SystemTime;
use ::std::time::UNIX_EPOCH;
use std::io::Read;

use ::chrono::{DateTime, Local};
use ::log::debug;
use ::memoize::memoize;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::fail;
use crate::common::Task;
use crate::wait::lockfile::{DATA_VERSION, Key, LockFile};

pub fn read(key: Key) -> Option<LockFile> {
    debug!("going to read lock for key '{}'", &key);
    let pth = lock_pth(key);
    if !pth.exists() {
        debug!("no lock file at '{}'", pth.to_string_lossy());
        return None
    }
    let content = read_file(&pth)?;
    let reader = BufReader::new(open_file(&pth, false));
    match bincode::from_reader::<_, TaskStack>(reader) {
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

// pub fn write(namespace: String, tasks: &TaskStack) {
//     debug!("going to write commands for namespace '{}'", &namespace);
//     let pth = lock_pth(namespace);
//     if tasks.is_empty() {
//         if pth.exists() {
//             debug!(
//                 "commands stack is empty, deleting commands file at '{}'",
//                 pth.to_string_lossy()
//             );
//             if let Err(err) = remove_file(&pth) {
//                 fail(&format!(
//                     "failed to remove commands in '{}', error: {}",
//                     pth.to_string_lossy(),
//                     err
//                 ));
//             }
//         } else {
//             debug!(
//                 "commands stack is empty, there is no commands file at '{}', doing nothing",
//                 pth.to_string_lossy()
//             );
//         }
//     } else {
//         let mut writer = BufWriter::new(open_file(&pth, true));
//         if let Err(err) = bincode::to_writer(&mut writer, tasks) {
//             fail(&format!(
//                 "failed to write commands in {}, error: {}",
//                 pth.to_string_lossy(),
//                 err
//             ));
//         }
//         writer.write_all(&[b'\n']).unwrap();
//         debug!(
//             "wrote updated commands file with {} commands to '{}'",
//             tasks.len(),
//             pth.to_string_lossy()
//         );
//     }
// }

#[memoize]
pub fn lock_pth(key: Key) -> PathBuf {
    let mut pth = make_app_dir();
    let filename = make_filename(key.clone());
    debug!("commands file for namespace '{}' and version {} is called '{}' inside cache directory '{}'",
            &key, DATA_VERSION, &filename, pth.to_string_lossy());
    pth.push(&filename);
    pth
}

fn make_app_dir() -> PathBuf {
    let mut pth = match dirs::cache_dir() {
        Some(pth) => pth,
        None => fail("failed to find cache directory"),
    };
    pth.push("locked");
    if let Err(err) = create_dir_all(&pth) {
        fail(format!(
            "failed to create directory {}, error {}",
            pth.to_string_lossy(),
            err
        ))
    }
    pth
}

fn make_filename(key: Key) -> String {
    let re = Regex::new("^[a-zA-Z0-9_-]+$").unwrap();
    if !re.is_match(&namespace) {
        fail("key should only contains alphanumeric characters, dashes and underscores");
    }
    format!(
        "lock_{}_v{}",
        key,
        DATA_VERSION
    )
}

fn read_file(pth: &Path) -> Result<Vec<u8>, String> {
    let mut opts = OpenOptions::new().read(true);
    match opts.open(pth) {
        Ok(mut file) => {
            let mut buf = vec![];
            file.read_to_end()?;
            Ok(buf)
        },
        Err(err) => {
            Err(format!(
                "failed to open lock file for reading '{}', error {}",
                pth.to_string_lossy(),
                err
            ))
        }
    }
}

// fn open_file(pth: &Path, write: bool) -> File {
//     let mut opts = OpenOptions::new();
//     if write {
//         opts.write(true).truncate(true).create(true)
//     } else {
//         opts.read(true)
//     };
//     match opts.open(pth) {
//         Ok(file) => file,
//         Err(err) => {
//             fail(&format!(
//                 "failed to open lock file at '{}' with options {:?}, error {}",
//                 pth.to_string_lossy(),
//                 &opts,
//                 err
//             ));
//         }
//     }
// }
//
// pub fn current_time_s() -> u32 {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_secs() as u32
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn default_lock_pth() {
//         assert_eq!(
//             lock_pth("".to_owned()).file_name().unwrap(),
//             format!("cmd_stack_v{}", DATA_VERSION).as_str()
//         );
//     }
//
//     #[test]
//     fn namespaced_lock_pth() {
//         assert_eq!(
//             lock_pth("1".to_owned()).file_name().unwrap(),
//             format!("cmd_stack_1_v{}", DATA_VERSION).as_str()
//         );
//     }
// }
