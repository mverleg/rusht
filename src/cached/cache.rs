use ::std::fs::{create_dir_all, OpenOptions};
use ::std::io::BufReader;
use ::std::path::PathBuf;

use ::chrono::{DateTime, Local};
use ::log::debug;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::cached::CachedArgs;
use crate::common::{fail, Task, unique_filename};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    RanSuccessfully(String),
    FromCache(String),
    Failed(i32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Cache {
    time: DateTime<Local>,
    task: Task,  // needed?
    output: String,
}

pub fn cached(args: CachedArgs) -> Result<CacheStatus, String> {
    //TODO @mark: duration
    let task = Task::new_split_in_cwd(args.cmd.unpack());
    let cache_pth = get_cache_path(&args.key, &task);
    let pth = &cache_pth;
    let write = false;
    let mut opts = OpenOptions::new();
    if write {
        opts.write(true).truncate(true).create(true)
    } else {
        opts.read(true)
    };
    let cache = OpenOptions::new().read(true)
        .open(pth)
        .map(|rdr| BufReader::new(rdr))
        .map(|rdr| serde_json::from_reader::<_, Cache>(rdr));
    match cache {
        Ok(Ok(cache)) => {
            debug!("found cached entry from {} at {}", &cache.time, cache_pth.to_string_lossy());
            let age = Local::now().signed_duration_since(cache.time).to_std().unwrap();
            if age > args.duration {
                debug!("cached entry is too old, {}s > {}s", &age.as_secs(), &args.duration.as_secs());
            } else {
                debug!("valid cache ({}s); was created with task: {}", age.as_secs(), cache.task.as_cmd_str());
                return Ok(CacheStatus::FromCache(cache.output))
            }
        }
        Ok(Err(_)) => {
            fail("failed to parse cache file");
        }
        Err(_) => {
            debug!("no cached entry at {}", cache_pth.to_string_lossy());
        }
    }
    task.execute(args.quiet);
    //TODO @mark: update cache
    unimplemented!()
}


fn get_cache_path(key_templ: &str, task: &Task) -> PathBuf {
    let key = key_templ
        .replace("${pwd}", task.working_dir.to_string_lossy().as_ref())
        .replace("${cmd}", &task.as_cmd_str());
    let filename = unique_filename(&key);
    let mut pth = dirs::cache_dir().expect("failed to find cache directory");
    pth.push("cmdcache");
    create_dir_all(&pth).unwrap();
    pth.push(filename);
    pth
}
