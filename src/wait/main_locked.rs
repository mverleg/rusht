use ::std::process::exit;

use ::clap::StructOpt;
use ::env_logger;

use ::rusht::wait::locked;
use ::rusht::wait::LockedArgs;

fn main() {
    env_logger::init();
    let args = LockedArgs::from_args();
    match locked(args) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("failed: {}", err);
            exit(1)
        }
    }
}
