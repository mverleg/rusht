use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;
use std::path::PathBuf;

const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
const MAIN_SRC: &str = include_str!("./template/src/main.rs");

pub fn compile_rsh(context: &RshContext, prog: RshProg) -> Result<PathBuf, String> {
    todo!();
}
