use ::std::path::PathBuf;

#[derive(Debug)]
pub struct RshProg {
    pub script_path: PathBuf,
    pub code: String,
}

impl RshProg {
    pub fn name(&self) -> String {
        let no_ext = self.script_path.with_extension("");
        no_ext
            .file_name()
            .expect("could not get filename")
            .to_str()
            .expect("filename is not utf8")
            .to_owned()
    }
}
