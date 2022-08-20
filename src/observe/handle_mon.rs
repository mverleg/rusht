use ::std::process::exit;

use crate::common::{StdinReader, StdoutWriter};
use crate::filter::filter;
use crate::filter::unique;
use crate::filter::FilterArgs;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

pub async fn handle_mon(args: MonArgs) {
    mon(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
}
