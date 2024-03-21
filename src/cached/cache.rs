use ::std::env;
use ::std::fs::create_dir_all;
use ::std::fs::OpenOptions;
use ::std::io::BufReader;
use ::std::io::Write;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::Duration;
use std::env::VarError;

use ::log::debug;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::time::OffsetDateTime;
use crate::cached::args::CachedKeyArgs;

use crate::cached::CachedArgs;
use crate::common::unique_filename;
use crate::common::fail;
use crate::common::git_head_ref;
use crate::common::git_master_base_ref;
use crate::common::LineWriter;
use crate::common::Task;
use crate::common::TeeWriter;
use crate::common::VecWriter;
use crate::ExitStatus;

pub const DATA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    RanSuccessfully,
    FromCache(String),
    Failed(ExitStatus),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Cache {
    time: OffsetDateTime,
    task: Task,
    output: String,
}

pub async fn cached(args: CachedArgs, writer: &mut impl LineWriter) -> Result<CacheStatus, String> {
    let task = args.cmd.clone().into_task();
    let cache_pth = get_cache_path(&args, &task)?;
    let cached_output = try_read_cache(&args.duration, &cache_pth);
    if let Some(output) = cached_output {
        return Ok(CacheStatus::FromCache(output));
    }
    let mut vec_writer = VecWriter::new();
    let mut tee_writer = TeeWriter::new(writer, &mut vec_writer);
    let exit_code = task
        .execute_with_stdout(args.verbose, &mut tee_writer)
        .await;
    if exit_code.is_err() {
        return Ok(CacheStatus::Failed(exit_code));
    }
    let output = vec_writer.get().join("\n");
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
            let age = OffsetDateTime::now_utc() - cache.time;
            if &age > max_age {
                debug!(
                    "cached entry is too old, {}s > {}s",
                    &age.whole_seconds(),
                    &max_age.as_secs()
                );
            } else {
                debug!(
                    "valid cache ({}s); was created with task: {}",
                    age.whole_seconds(),
                    cache.task.as_str()
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
        time: OffsetDateTime::now_utc(),
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

fn get_cache_path(args: &CachedArgs, task: &Task) -> Result<PathBuf, String> {
    let key = build_key(args, task)?;
    let filename = unique_filename(&key);
    let mut pth = dirs::cache_dir().expect("failed to find cache directory");
    pth.push(format!("cmdcache_v{}", DATA_VERSION));
    create_dir_all(&pth).unwrap();
    pth.push(filename);
    debug!("created cache path {} from args {:?}", pth.to_string_lossy(), args);
    Ok(pth)
}

fn build_key(args: &CachedArgs, task: &Task) -> Result<String, String> {
    build_key_with(&args.key, task, read_from_sys_env)
}

fn read_from_sys_env(env_key: &str) -> Result<String, String> {
    Ok(match env::var(env_key) {
        Ok(val) => format!("{env_key}-{val}"),
        Err(VarError::NotPresent) => format!("{env_key}_NO"),
        Err(VarError::NotUnicode(_)) =>
            return Err(format!("cannot cache env '{env_key}' because value is not unicode")),
    })
}

fn build_key_with(
    args: &CachedKeyArgs,
    task: &Task,
    get_from_env: impl Fn(&str) -> Result<String, String>
) -> Result<String, String> {
    assert!(!args.git_pending, "--git-pending not implemented");
    debug_assert!(args.env.is_sorted());
    debug_assert!(args.env.is_sorted());
    let mut key: Vec<String> = Vec::new();
    if ! args.no_dir {
        key.push(task.working_dir.to_string_lossy().into_owned())
    }
    if ! args.no_command {
        key.push(task.as_cmd_str())
    }
    if ! args.no_direct_env {
        for (env_key, value) in &task.extra_envs {
            key.push(format!("{}-{}", env_key, value))
        }
    }
    if args.git_head {
        let head = git_head_ref(&task.working_dir).map_err(|err| {
            format!("caching based on git HEAD, but could not read it, err: {err}") })?;
        key.push(head)
    } else if args.git_base {
        let head = git_master_base_ref(&task.working_dir).map_err(|err| {
            format!("caching based on git merge base, but could not determine it, err: {err}") })?;
        key.push(head)
    }
    for env_key in &args.env {
        key.push(get_from_env(env_key)?)
    }
    for text in &args.text {
        key.push(text.to_owned())
    }
    Ok(unique_filename(&key.join("_")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    fn create_test_task() -> Task {
        Task {
            cmd: "ls".to_owned(),
            args: vec!["-a".to_owned()],
            working_dir: PathBuf::from("/tmp"),
            extra_envs: HashMap::new(),
            stdin: None,
        }
    }

    fn read_from_test_env(env_key: &str) -> Result<String, String> {
        Ok(match env_key {
            "KEY" => "ENV_VALUE".to_owned(),
            other_key => format!("{other_key}_NO"),
        })
    }

    #[test]
    fn build_key_vanilla() {
        let task = create_test_task();
        let args = CachedArgs::default();
        let key = build_key_with(&args.key, &task, read_from_test_env);
        assert_eq!(key, Ok("tmp_ls_a_qjtza8xbfyol".to_owned()));
    }

    #[test]
    fn build_key_with_text_env() {
        let task = create_test_task();
        let args = CachedArgs {
            key: CachedKeyArgs {
                text: vec!["hello".to_owned(), "world".to_owned()],
                env: vec!["VAR".to_owned()],
                ..Default::default()
            },
            ..Default::default()
        };
        let key = build_key_with(&args.key, &task, read_from_test_env);
        assert_eq!(key, Ok("tmp_ls_a_VAR_NO_hellq1kzva1h4vlt".to_owned()));
    }
}
