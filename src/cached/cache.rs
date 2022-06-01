use crate::cached::CachedArgs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheString {
    RanSuccessfully,
    FromCache,
    Failed(u8),
}

pub fn cached(_args: CachedArgs) -> Result<CacheString, String> {
    //TODO @mark: The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path.

    unimplemented!()
}
