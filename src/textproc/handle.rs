use crate::common::{StdinReader, StdWriter};
use crate::ExitStatus;
use crate::textproc::batched::batched;
use crate::textproc::batched_args::BatchedArgs;

pub async fn handle_batched(args: BatchedArgs) -> ExitStatus {
    batched(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
