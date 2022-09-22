use std::path::PathBuf;

#[derive(Debug)]
pub struct RshProg {
    pub path: PathBuf,
    pub code: String,
}

impl RshProg {
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .expect("could not get filename")
            .to_str()
            .expect("filename is not utf8")
    }
}
