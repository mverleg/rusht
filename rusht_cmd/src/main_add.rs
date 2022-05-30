
use ::structopt::StructOpt;
use ::rusht_cmd::AddArgs;
use ::rusht_cmd::add_cmd;

//TODO: option to deduplicate tasks
//TODO: run inside Docker?
//TODO: source bashrc/profile
//TODO: set default command for when stack is empty

fn main() {
    env_logger::init();
    let mut args = AddArgs::from_args();
    if args.lines {
        args.lines_with = Some("{}".to_owned());
    }
    assert!(!args.skip_validation, "skip_validation not implemented");
    add_cmd(args);
}
