use crate::common::{StdWriter, StdinReader};
use crate::jl::jl::list_files;
use crate::jl::jl_args::JlArgs;
use crate::ExitStatus;

pub async fn handle_jl(args: JlArgs) -> ExitStatus {
    list_files(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
}
