use ::std::fs::{create_dir_all, OpenOptions};
use ::std::io::BufReader;
use ::std::io::Write;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::Duration;

use ::chrono::{DateTime, Local};
use ::log::debug;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::cached::CachedArgs;
use crate::common::{fail, unique_filename, Task};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    RanSuccessfully,
    FromCache(String),
    Failed(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Cache {
    time: DateTime<Local>,
    task: Task,
    output: String,
}

pub fn cached(args: CachedArgs) -> Result<CacheStatus, String> {
    let task = Task::new_split_in_cwd(args.cmd.unpack());
    let cache_pth = get_cache_path(&args.key, &task);
    let cached_output = try_read_cache(&args.duration, &cache_pth);
    if let Some(output) = cached_output {
        return Ok(CacheStatus::FromCache(output));
    }
    let mut output = String::new();
    let exit_code = task.execute_with_stdout(!args.verbose, |line| {
        print!("{}", line);
        output.push_str(line);
    });
    if !exit_code.success() {
        return Ok(CacheStatus::Failed(exit_code.code().unwrap_or(1)));
    }
    update_cache(output, task, &cache_pth);
    Ok(CacheStatus::RanSuccessfully)
}

fn try_read_cache(max_age: &Duration, cache_pth: &Path) -> Option<String> {
    let cache = OpenOptions::new()
        .read(true)
        .open(cache_pth)
        .map(BufReader::new)
        .map(serde_json::from_reader::<_, Cache>);
    match cache {
        Ok(Ok(cache)) => {
            debug!(
                "found cached entry from {} at {}",
                &cache.time,
                cache_pth.to_string_lossy()
            );
            let age = Local::now()
                .signed_duration_since(cache.time)
                .to_std()
                .unwrap();
            if &age > max_age {
                debug!(
                    "cached entry is too old, {}s > {}s",
                    &age.as_secs(),
                    &max_age.as_secs()
                );
            } else {
                debug!(
                    "valid cache ({}s); was created with task: {}",
                    age.as_secs(),
                    cache.task.as_cmd_str()
                );
                return Some(cache.output);
            }
        }
        Ok(Err(_)) => {
            fail("failed to parse cache file");
        }
        Err(_) => {
            debug!("no cached entry at {}", cache_pth.to_string_lossy());
        }
    }
    None
}

fn update_cache(output: String, task: Task, cache_pth: &Path) {
    let mut cache_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(cache_pth)
        .unwrap_or_else(|err| {
            panic!(
                "failed to create/open cache file at {}, err {}",
                cache_pth.to_string_lossy(),
                err
            )
        });
    let cache = Cache {
        time: Local::now(),
        task,
        output,
    };
    let cache_json = serde_json::to_string(&cache).expect("failed to serialize cache");
    debug!(
        "writing output ({} bytes json) to cache at {}",
        cache_json.len(),
        cache_pth.to_string_lossy()
    );
    cache_file
        .write_all(cache_json.as_bytes())
        .unwrap_or_else(|err| {
            panic!(
                "failed to write to cache file at {}, err {}",
                cache_pth.to_string_lossy(),
                err
            )
        });
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
