use crate::rscript::rsh_program::RshProg;

const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
const MAIN_SRC: &str = include_str!("./template/src/main.rs");

pub fn compile_rsh(prog: RshProg) -> Result<(), String> {
    todo!();
}
