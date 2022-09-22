#![allow(unused)]

use ::clap::StructOpt;
use ::num_cpus;

include!("./args.rs");
include!("./run.rs");

pub fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = Args::from_args();
    run(args);
}
