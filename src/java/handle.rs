use crate::common::{StdinReader, StdoutWriter};
use crate::java::MvnwArgs;
use crate::java::mvnw;

pub async fn handle_mvnw(args: MvnwArgs) {
    mvnw(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
}
