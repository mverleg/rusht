use crate::common::StdoutWriter;
use crate::java::mvnw;
use crate::java::MvnwArgs;
use crate::ExitStatus;

pub async fn handle_mvnw(args: MvnwArgs) -> ExitStatus {
    match mvnw(args, &mut StdoutWriter::new()).await {
        Ok(()) => ExitStatus::ok(),
        Err((code, err_msg)) => {
            if !err_msg.is_empty() {
                eprintln!("{}", err_msg);
            }
            code
        }
    }
}
