use ::std::io;

use ::log::debug;
use ::walkdir::DirEntry;
use ::walkdir::WalkDir;

use crate::common::LineWriter;
use crate::ExitStatus;
use crate::find::jl_args::{ErrorHandling, JlArgs};
use crate::find::jl_json_api::FSNode;

pub async fn list_files(
    args: JlArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.no_recurse_symlinks, "no_recurse_symlinks not impl");
    assert!(!args.entry_per_lines, "entry_per_lines not impl");

    //TODO @mverleg: filter
    //TODO @mverleg: root
    let mut has_err = false;
    let mut is_first = true;  //TODO @mverleg:
    let mut line = String::new();
    if ! args.entry_per_lines {
        line.push('[');
    }
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
        let name = file.path().display();  //TODO @mverleg: TEMPORARY! REMOVE THIS!
        let node = FSNode {

        };
        serde_json::to_writer(io::Cursor(&mut line), &node).expect("failed to create json from FSNode");
        writer.write_line(&line).await;
        line.clear();
    }
    if ! args.entry_per_lines {
        line.push(']');
    }
    writer.write_line(&line).await;
    assert!(!has_err);  //TODO @mverleg: msg
    ExitStatus::ok()
}
