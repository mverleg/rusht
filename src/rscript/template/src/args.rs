
#[derive(StructOpt, Debug, Clone)]
#[structopt(about = env::var("CARGO_PKG_DESCRIPTION").unwrap())]
pub struct Args {
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    Args::into_app().debug_assert()
}
