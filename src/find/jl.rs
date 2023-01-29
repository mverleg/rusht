use ::log::debug;

use ::walkdir::DirEntry;
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
    let mut has_err = false;
    //TODO @mverleg: async walk dir?
    for file_res in WalkDir::new(args.root).into_iter() {
        let file: DirEntry = match file_res {
            Ok(file) => file,
            Err(err) => {
                match args.on_error {
                    ErrorHandling::Abort => {
                        eprintln!("failed to read file, error: {err}");
                        return ExitStatus::of(1)
                    },
                    ErrorHandling::FailAtEnd => { has_err = true; }
                    ErrorHandling::Warn => eprintln!("failed to read file, error: {err}"),
                    ErrorHandling::Ignore => debug!("ignoring file read error: {err}"),
                }
                continue
            }
        };
        let name = file.path().display();
        writer.write_line(name).await
    }

    todo!();
}
