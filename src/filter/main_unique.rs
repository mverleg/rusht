use ::clap::StructOpt;
use ::ustr::Ustr;

use ::rusht::common::{EmptyLineHandling, stdin_lines};
use ::rusht::filter::{unique, unique_prefix, UniqueArgs};
use ::rusht::filter::handle_unique;

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    handle_unique(args)
}
