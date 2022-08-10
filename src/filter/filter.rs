use ::std::collections::HashSet;
use ::std::env::current_dir;
use ::std::io::Read;
use ::std::io::stdin;
use ::std::path::PathBuf;
use ::std::thread::spawn;

use ::clap::StructOpt;
use ::log::debug;

use crate::common::{CommandArgs, fail, LineReader, LineWriter, Task};
use crate::filter::FilterArgs;

pub async fn unique(args: FilterArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) {

}
