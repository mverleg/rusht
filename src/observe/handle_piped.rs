use crate::common::{StdWriter, StdinReader};
use crate::observe::piped::piped;
use crate::observe::piped_args::PipedArgs;
use crate::ExitStatus;

pub async fn handle_piped(args: PipedArgs) -> ExitStatus {
    piped(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
