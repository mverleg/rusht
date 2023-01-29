use crate::common::{LineReader, LineWriter};
use crate::jl::jl_args::JlArgs;
use crate::ExitStatus;

pub async fn list_files(
    args: JlArgs,
    reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.no_recurse_symlinks, "no_recurse_symlinks not impl");
    assert!(!args.entry_per_lines, "entry_per_lines not impl");
    todo!()
}
