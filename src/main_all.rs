use ::clap::Parser;
use ::clap::Subcommand;
use ::env_logger;

use ::rusht::cached::CachedArgs;
use ::rusht::cached::handle_cached;
use ::rusht::cmd::{handle_add, handle_do, handle_drop, handle_list};
use ::rusht::cmd::{AddArgs, DoArgs, DropArgs, ListArgs};
use ::rusht::escape::handle_namesafe;
use ::rusht::escape::NamesafeArgs;
use ::rusht::ExitStatus;
use ::rusht::filter::{FilterArgs, handle_filter};
use ::rusht::filter::{handle_grab, handle_unique};
use ::rusht::filter::{GrabArgs, UniqueArgs};
use ::rusht::find::DirWithArgs;
use ::rusht::find::handle_dir_with;
use ::rusht::find::JlArgs;
use ::rusht::java::{handle_mvnw, MvnwArgs};
use ::rusht::observe::{handle_mon, MonArgs};
use ::rusht::observe::{handle_piped, PipedArgs};
use ::rusht::rsh::{handle_rsh, RshArgs};
use ::rusht::wait::handle_locked;
use ::rusht::wait::LockedArgs;
use rusht::find::handle_jl;
use rusht::textproc::batched_args::BatchedArgs;
use rusht::textproc::handle::handle_batched;

#[derive(Parser, Debug)]
#[command(
    name = "rusht",
    about = "Single executable with all rusht utilities as subcommands."
)]
pub struct RushtArgs {
    #[clap(subcommand)]
    subcommand: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    Cached(CachedArgs),
    Cmadd(AddArgs),
    Cmdo(DoArgs),
    Cmlist(ListArgs),
    Cmdrop(DropArgs),
    //TODO @mverleg: get this to use dir_with
    DirWith(DirWithArgs),
    Grab(GrabArgs),
    Unique(UniqueArgs),
    Filter(FilterArgs),
    Locked(LockedArgs),
    Namesafe(NamesafeArgs),
    Mvnw(MvnwArgs),
    Mon(MonArgs),
    Piped(PipedArgs),
    Batched(BatchedArgs),
    Jl(JlArgs),
    Rsh(RshArgs),
}

#[test]
fn test_cli_args() {
    RushtArgs::try_parse_from(&["cmd", "cached", "echo", "hi"]).unwrap();
}

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = RushtArgs::parse();
    match args.subcommand {
        SubCmd::Cached(sub_args) => handle_cached(sub_args).await,
        SubCmd::Cmadd(sub_args) => handle_add(sub_args),
        SubCmd::Cmdo(sub_args) => handle_do(sub_args),
        SubCmd::Cmlist(sub_args) => handle_list(sub_args),
        SubCmd::Cmdrop(sub_args) => handle_drop(sub_args),
        SubCmd::DirWith(sub_args) => handle_dir_with(sub_args),
        SubCmd::Grab(sub_args) => handle_grab(sub_args).await,
        SubCmd::Unique(sub_args) => handle_unique(sub_args).await,
        SubCmd::Filter(sub_args) => handle_filter(sub_args).await,
        SubCmd::Locked(sub_args) => handle_locked(sub_args),
        SubCmd::Namesafe(sub_args) => handle_namesafe(sub_args),
        SubCmd::Mvnw(sub_args) => handle_mvnw(sub_args).await,
        SubCmd::Mon(sub_args) => handle_mon(sub_args).await,
        SubCmd::Piped(sub_args) => handle_piped(sub_args).await,
        SubCmd::Batched(sub_args) => handle_batched(sub_args).await,
        SubCmd::Jl(sub_args) => handle_jl(sub_args).await,
        SubCmd::Rsh(sub_args) => handle_rsh(sub_args),
    }
}
