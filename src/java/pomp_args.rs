use ::std::path::PathBuf;

use ::clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pomp",
    about = "Parse a tiny subset of pomfiles.",
)]
pub struct PompArgs {
    /// Pomfile paths
    #[arg(required = true)]
    pub pom_paths: Vec<PathBuf>,
    #[arg(short='a', long="artifactId")]
    pub artifact_id: bool,
    #[arg(short='g', long="groupId")]
    pub group_id: bool,
    #[arg(short='v', long="version")]
    pub version: bool,
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
