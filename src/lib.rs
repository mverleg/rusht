use ::std::path::PathBuf;

use ::async_walkdir::WalkDir;
use ::futures_lite::stream::StreamExt;

pub async fn for_all_files<T, E>(mut op: impl FnMut(PathBuf) -> Result<T, E>) -> Result<Vec<T>, E> {
    let mut entries = WalkDir::new(".");
    let mut results = vec![];
    loop {
        match entries.next().await {
            Some(Ok(entry)) => {
                if entry.metadata().i {
                    results.push(op(entry.path())?)
                }
            },
            Some(Err(err)) => panic!("walkdir error: {}", err),
            None => break,
        }
    }
    Ok(results)
}

pub async fn for_unignored_files<T, E>(_op: impl FnMut(PathBuf) -> Result<T, E>) -> Result<Vec<T>, E> {
    panic!();  //TODO @mark: TEMPORARY! REMOVE THIS!
}
