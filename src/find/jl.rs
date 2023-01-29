use ::walkdir::WalkDir;

use crate::common::LineWriter;
use crate::ExitStatus;
use crate::find::jl_args::{ErrorHandling, JlArgs};

pub async fn list_files(
    args: JlArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.no_recurse_symlinks, "no_recurse_symlinks not impl");
    assert!(!args.entry_per_lines, "entry_per_lines not impl");

    //TODO @mverleg: filter
    //TODO @mverleg: root
    for file_res in WalkDir::new(args.root).into_iter() {
        let file = match file_res {
            Ok(file) => file,
            Err(_) => match args.on_error {
                ErrorHandling::Abort => todo!(),
                ErrorHandling::FailAtEnd => todo!(),
                ErrorHandling::Warn => todo!(),
                ErrorHandling::Ignore => todo!(),
            }
        };
        writer.write_line(file.path().display()).await
    }

    todo!();
}
