use crate::common::{LineReader, LineWriter};
use crate::ExitStatus;
use crate::find::jl_args::JlArgs;

pub async fn list_files(
    args: JlArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.no_recurse_symlinks, "no_recurse_symlinks not impl");
    assert!(!args.entry_per_lines, "entry_per_lines not impl");

    for file in WalkDir::new("./change_this_path").into_iter().filter_map(|file| file.ok()) {
        println!("{}", file.path().display());
    }

    let key = get_first_match_or_all(unique_by_pattern, line);
    if !keep.keep_is_first(seen.insert(key.to_owned())) {
        continue;
    }
    writer.write_line(line).await
}
