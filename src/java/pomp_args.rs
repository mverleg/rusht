use ::std::path::PathBuf;

use ::clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pomp",
    about = "Parse a tiny subset of pomfiles.",
)]
pub struct PompArgs {
    /// Pomfile paths
    #[arg()]
    pub proj_roots: Vec<PathBuf>,
    #[arg(short='a', long="artifactId")]
    artifact_id: bool,
    #[arg(short='g', long="groupId")]
    group_id: bool,
    #[arg(short='v', long="version")]
    version: bool,
}

#[test]
fn test_cli_args() {
    PompArgs::try_parse_from(&[
        "cmd",
        "--artifactId",
        "-gv",
        "module/pom.xml",
        "module2/pom.xml",
    ])
    .unwrap();
}
