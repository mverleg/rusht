use crate::common::{StdinReader, StdoutWriter};
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn handle_mon(args: MonArgs) -> ExitStatus {
    mon(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await
}
