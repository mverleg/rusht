use crate::common::LineWriter;
use crate::java::PompArgs;
use crate::ExitStatus;
use ::serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Pom<'a> {
    artifactId: &'a str,
    groupId: &'a str,
    version: &'a str,
}

pub fn pomp(
    args: PompArgs,
    writer: &mut impl LineWriter,
) -> Result<(), (ExitStatus, String)> {
    for pth in args.pom_paths {
        if ! pth.is_file() {
            return Err((ExitStatus::err(), format!("{} is not a pomfile", pth.display())))
        }

    }
    todo!();
    Ok(())
}
