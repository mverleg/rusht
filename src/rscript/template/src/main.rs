#![allow(unused)]

use ::clap::StructOpt;
use ::num_cpus;
use ::std::env;
use ::std::env::var;

include!("./args.rs");
include!("./run.rs");

pub fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    if var("RSH_NAME").is_err() {
        eprintln!("it is recommended to run through 'rsh', like 'rsh {}', instead of calling '{}' directly");
    }
    let args = Args::from_args();
    run(args);
}
