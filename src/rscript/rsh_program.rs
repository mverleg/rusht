use std::path::PathBuf;

#[derive(Debug)]
pub struct RshProg {
    pub path: PathBuf,
    pub code: String,
}

impl RshProg {
    pub fn name(&self) -> &str {
        self.path.to_str().unwrap()
    }
}
