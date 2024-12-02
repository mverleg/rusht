use crate::common::StdWriter;
use crate::java::{mvnw, PompArgs};
use crate::java::MvnwArgs;
use crate::ExitStatus;

pub async fn handle_mvnw(args: MvnwArgs) -> ExitStatus {
    match mvnw(args, &mut StdWriter::stdout()).await {
        Ok(()) => ExitStatus::ok(),
        Err((code, err_msg)) => {
            if !err_msg.is_empty() {
                eprintln!("{}", err_msg);
            }
            code
        }
    }
}

pub fn handle_pomp(args: PompArgs) -> ExitStatus {

}
