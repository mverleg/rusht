
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "rsh-generated", about = "This is just a placeholder")]
pub struct Args {
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    Args::into_app().debug_assert()
}
