use std::path::PathBuf;

#[derive(Debug)]
pub struct RshProg {
    pub path: PathBuf,
    pub code: String,
}
