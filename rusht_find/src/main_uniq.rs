use ::std::io;

fn main() {
    env_logger::init();
    let args = UniqArgs::from_args();
    let lines = stdin.lock().lines().collect();
    unique(args, lines);
}
