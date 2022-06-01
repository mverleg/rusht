use ::std::env::current_dir;
use ::std::path::PathBuf;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::cached::CachedArgs;
use crate::common::Task;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    RanSuccessfully,
    FromCache,
    Failed(u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Cache {
    timestamp: u64,
    task: Task,  // needed?
    output: String,
}

pub fn cached(args: CachedArgs) -> Result<CacheStatus, String> {
    //TODO @mark: The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path.

    //TODO @mark: duration
    //TODO @mark: key

    let task = Task::new_split_in_cwd(args.cmd.unpack());
    let cache_pth = get_cache_path(&args.key, &task);
    eprintln!("cache not ready; always running");  //TODO @mark: TEMPORARY! REMOVE THIS!
    task.execute(false);
}

fn get_cache_path(key_templ: &str, task: &Task) -> PathBuf {
    unimplemented!()
}