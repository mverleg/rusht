use ::log::debug;

use crate::common::StdWriter;
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

pub async fn handle_mon(args: MonArgs) -> ExitStatus {
    if args.use_stdout {
        debug!("use `mon` with monitor lines logged to stdout");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stdout()).await
    } else {
        debug!("use `mon` with monitor lines logged to stderr");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stderr()).await
    }
}
