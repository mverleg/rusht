use crate::common::StdWriter;
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn handle_mon(args: MonArgs) -> ExitStatus {
    if args.use_stderr {
        mon(args, &mut StdWriter::stderr()).await
    } else {
        mon(args, &mut StdWriter::stdout()).await
    }
}
