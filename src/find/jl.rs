use crate::common::{LineReader, LineWriter};
use crate::ExitStatus;
use crate::find::jl_args::JlArgs;

pub async fn list_files(
    args: JlArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.no_recurse_symlinks, "no_recurse_symlinks not impl");
    assert!(!args.entry_per_lines, "entry_per_lines not impl");
    todo!()
}
