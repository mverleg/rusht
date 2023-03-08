use ::log::debug;

use crate::common::StdWriter;
use crate::ExitStatus;
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;

pub async fn handle_mon(args: MonArgs) -> ExitStatus {
    if args.use_stderr {
        debug!("use `mon` with monitor lines logged to stderr");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stderr()).await
    } else {
        debug!("use `mon` with monitor lines logged to stdou");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stdout()).await
    }
}
