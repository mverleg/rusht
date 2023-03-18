use crate::common::{StdWriter, StdinReader};
use crate::textproc::batched::batched;
use crate::textproc::batched_args::BatchedArgs;
use crate::ExitStatus;

pub async fn handle_batched(args: BatchedArgs) -> ExitStatus {
    match batched(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{err}");
            ExitStatus::err()
        }
    }
}
