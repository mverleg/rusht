use crate::common::{StdinReader, StdWriter};
use crate::ExitStatus;
use crate::observe::piped::piped;
use crate::observe::piped_args::PipedArgs;

pub async fn handle_piped(args: PipedArgs) -> ExitStatus {
    piped(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
