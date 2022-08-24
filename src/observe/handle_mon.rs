use crate::common::{StdinReader, StdWriter};
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn handle_mon(args: MonArgs) -> ExitStatus {
    mon(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
