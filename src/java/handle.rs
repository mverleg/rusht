use crate::common::StdoutWriter;
use crate::java::mvnw;
use crate::java::MvnwArgs;

pub async fn handle_mvnw(args: MvnwArgs) {
    mvnw(args, &mut StdoutWriter::new()).await;
}
