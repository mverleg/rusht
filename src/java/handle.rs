use ::std::process::exit;

use crate::common::StdoutWriter;
use crate::java::mvnw;
use crate::java::MvnwArgs;

pub async fn handle_mvnw(args: MvnwArgs) {
    match mvnw(args, &mut StdoutWriter::new()).await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
