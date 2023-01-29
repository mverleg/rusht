use crate::common::{StdWriter, StdinReader};
use crate::jl::jl::piped;
use crate::observe::PipedArgs;
use crate::ExitStatus;

pub async fn handle_piped(args: JlArgs) -> ExitStatus {
    jl(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
