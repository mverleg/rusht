use ::std::process::exit;

use crate::common::StdoutWriter;
use crate::java::mvnw;
use crate::java::MvnwArgs;

pub async fn handle_mvnw(args: MvnwArgs) {
    match mvnw(args, &mut StdoutWriter::new()).await {
        Ok(()) => {}
        Err((code, err_msg)) => {
            if ! err_msg.is_empty() {
                eprintln!("{}", err_msg);
            }
            exit(if code > 0 { code } else { 1 });
        }
    }
}
