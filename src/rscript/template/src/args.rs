
#[derive(StructOpt, Debug, Clone)]
#[structopt()]
pub struct Args {
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    Args::into_app().debug_assert()
}
