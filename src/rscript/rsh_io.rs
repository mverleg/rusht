use ::std::fs::read_to_string;
use ::std::path::Path;

use ::log::debug;

pub fn load_source(path: &Path) -> Result<String, String> {
    let mut src = read_to_string(path).map_err(|err| {
        format!(
            "failed to read rsh source file at '{}', err {}",
            path.to_string_lossy(),
            err
        )
    })?;
    if src.starts_with("#!") && !src.starts_with("#![") {
        debug!("detected shebang, stripping first line");
        //TODO @mverleg: deal with different platform line breaks?
        src = match src.split_once("\n") {
            Some((_, content)) => content.to_owned(),
            None => "".to_owned(),
        };
    }
    debug!(
        "rsh sript at '{}' with {} bytes",
        path.to_string_lossy(),
        src.len()
    );
    Ok(src)
}
