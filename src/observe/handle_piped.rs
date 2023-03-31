use crate::common::StdWriter;
use crate::common::StdinReader;
use crate::observe::piped::piped;
use crate::observe::piped_args::PipedArgs;
use crate::ExitStatus;

pub async fn handle_piped(args: PipedArgs) -> ExitStatus {
    if args.stderr {
        piped(args, &mut StdinReader::new(), &mut StdWriter::stderr()).await
    } else {
        piped(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
    }
}
