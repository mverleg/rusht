use ::serde::Deserialize;
use ::serde::Serialize;

use crate::cached::CachedArgs;
use crate::common::Task;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheString {
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

pub fn cached(_args: CachedArgs) -> Result<CacheString, String> {
    //TODO @mark: The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path.

    unimplemented!()
}
