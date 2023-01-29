use crate::common::{LineReader, LineWriter};
use std::process::ExitStatus;

pub async fn piped(
    args: JlArgs,
    _reader: &mut impl LineReader,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    todo!()
}
