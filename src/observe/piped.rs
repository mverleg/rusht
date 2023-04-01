#![allow(unused)]  //TODO @mark: TEMPORARY! REMOVE THIS!

use ::std::future::join;

use crate::common::LineReader;
use crate::common::LineWriter;
use crate::common::StdWriter;
use crate::ExitStatus;
use crate::observe::chained;
use crate::observe::piped_args::PipedArgs;

pub async fn piped(
    args: PipedArgs,
    outer_reader: &mut impl LineReader,
    outer_writer: &mut impl LineWriter,
) -> ExitStatus {
    assert!(!args.stderr);
    let (source_cmd, sink_cmd) = args.cmds.split_once_at(&args.separator);
    let buffer_size = args.pipe_buffer_size.try_into().unwrap_or(usize::MAX);
    let (mut chain_write, chain_read) = chained(buffer_size);
    unimplemented!();  //TODO @mark:
    // let (source_res, sink_res) = join!(
    //     source_cmd.into_task().execute_with_stdout_nomonitor(outer_reader, chain_write, &mut StdWriter::stderr()),
    //     sink_cmd.into_task().execute_with_stdout_nomonitor(chain_read, outer_writer, &mut StdWriter::stderr()),
    // ).await;
    // ExitStatus::max(source_res, sink_res)
}

#[cfg(test)]
mod tests {
    use crate::common::CollectorWriter;
    use crate::common::CommandArgs;
    use crate::common::VecReader;

    use super::*;

    #[async_std::test]
    async fn test_add() {
        let mut writer = CollectorWriter::new();
        let args = PipedArgs {
            separator: "//".to_string(),
            stderr: false,
            pipe_buffer_size: 4,
            cmds: CommandArgs::Cmd(vec![
                "echo".to_owned(),
                "-n".to_owned(),
                "hello world\nhow are you".to_owned(),
                "//".to_owned(),
                "wc".to_owned(),
                "-l".to_owned(),
            ]),
        };
        let res = piped(
            args,
            &mut VecReader::new(vec!["ignore this input"]),
            &mut writer
        ).await;
        assert!(res.is_ok());
        let output = writer.lines().snapshot().await.clone();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], "2");
    }
}
