use crate::common::StdWriter;
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn handle_piped(args: MonArgs) -> ExitStatus {
    piped(args, &mut StdWriter::stdout()).await
}
