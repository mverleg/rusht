use ::std::env;
use ::std::path::PathBuf;

use ::log::debug;

use crate::common::unique_filename;

static CACHE_DIR_ENV: &str = "RSH_CACHE_DIR";

#[derive(Debug)]
pub struct RshContext {
    cache_dir: PathBuf,
}

impl RshContext {
    pub fn empty_template_dir(&self) -> PathBuf {
        let mut pth = self.cache_dir.clone();
        pth.push("rsh_template");
        pth
    }

    pub fn state_path_for(&self, name: &str) -> PathBuf {
        let mut pth = self.cache_dir.clone();
        pth.push("rsh_state");
        pth.push(format!("{}.json", unique_filename(name)));
        pth
    }
}

pub fn rsh_context() -> Result<RshContext, String> {
    let build_dir = env::var(CACHE_DIR_ENV)
        .map(|pth| PathBuf::from(pth))
        .unwrap_or_else(|_| {
            let mut pth = dirs::cache_dir().expect("failed to find cache directory");
            pth.push("rsh_build");
            pth
        });
    debug!(
        "cache dir: '{}' (controlled by ${})",
        build_dir.to_string_lossy(),
        CACHE_DIR_ENV
    );
    Ok(RshContext {
        cache_dir: build_dir,
    })
}
