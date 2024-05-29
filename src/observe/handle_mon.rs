use std::env;
use std::env::VarError;
use ::log::debug;

use crate::common::StdWriter;
use crate::observe::mon::mon;
use crate::observe::mon_args::MonArgs;
use crate::ExitStatus;

static MON_FULL_CMD_VAR_NAME: &'static str = "MON_FULL_CMD";

pub async fn handle_mon(mut args: MonArgs) -> ExitStatus {
    update_full_cmd_flag_from_env(&mut args);
    if args.use_stdout {
        debug!("use `mon` with monitor lines logged to stdout");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stdout()).await
    } else {
        debug!("use `mon` with monitor lines logged to stderr");
        mon(args, &mut StdWriter::stdout(), &mut StdWriter::stderr()).await
    }
}

fn update_full_cmd_flag_from_env(args: &mut MonArgs) {
    if !args.full_command {
        match env::var(MON_FULL_CMD_VAR_NAME) {
            Ok(val) => if !val.trim().is_empty() && val != "0" {
                debug!("showing full `mon` command because env {MON_FULL_CMD_VAR_NAME} is set");
                args.full_command = true;
            }
            Err(VarError::NotPresent) => {}
            Err(_) => panic!("cannot read var {MON_FULL_CMD_VAR_NAME}"),
        }
    }
}
