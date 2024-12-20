use crate::common::LineWriter;
use crate::java::PompArgs;
use crate::ExitStatus;
use ::async_std::task::block_on;
use ::serde::Deserialize;
use ::serde_xml_rs::from_str;
use ::std::fs;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Pom {
    parent: Option<ArtifactId>,
    //TODO: cannot get #[serde(flatten)] to work
    artifactId: String,
    groupId: Option<String>,
    version: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ArtifactId {
    groupId: Option<String>,
    version: Option<String>,
}

pub fn pomp(
    args: PompArgs,
    writer: &mut impl LineWriter,
) -> Result<(), (ExitStatus, String)> {
    if !args.artifact_id && !args.group_id && !args.version {
        return Err((ExitStatus::err(), "Should request at least one output, e.g. one or more of -agv".to_owned()))
    }
    for pth in args.pom_paths {
        if ! pth.is_file() {
            return Err((ExitStatus::err(), format!("Not a (pom)file: {}", pth.display())))
        }
        let Ok(contents) = fs::read_to_string(&pth) else {
            return Err((ExitStatus::err(), format!("Cannot read utf8 from pomfile {}",pth.display())))
        };
        let pom = match from_str::<Pom>(&contents) {
            Ok(p) => p,
            Err(err) => return Err((ExitStatus::err(), format!("Cannot parse xml in {}, error: {}",pth.display(), err))),
        };
        let mut parts = Vec::with_capacity(3);
        if args.group_id {
            let parent_fallback = pom.parent.as_ref().map(|parent| parent.groupId.as_ref()).flatten();
            parts.push(match (pom.groupId, parent_fallback) {
                (Some(part), _) => part.to_owned(),
                (None, Some(parent)) => parent.to_owned(),
                (None, None) => return Err((ExitStatus::err(), format!("No groupId and no parent groupId for {}",pth.display()))),
            })
        }
        if args.artifact_id {
            parts.push(pom.artifactId.clone())
        }
        if args.version {
            let parent_fallback = pom.parent.as_ref().map(|parent| parent.version.as_ref()).flatten();
            parts.push(match (pom.version, parent_fallback) {
                (Some(part), _) => part.to_owned(),
                (None, Some(parent)) => parent.to_owned(),
                (None, None) => return Err((ExitStatus::err(), format!("No version and no parent version for {}",pth.display()))),
            })
        }
        let line = parts.join(":");
        block_on(writer.write_line(line));
    }
    Ok(())
}
