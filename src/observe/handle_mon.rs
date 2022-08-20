use ::std::process::exit;

use crate::common::{StdinReader, StdoutWriter};
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;

pub async fn handle_mon(args: MonArgs) {
    match mon(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await {
        Ok(()) => {}
        Err(code) => exit(code),
    }
}
