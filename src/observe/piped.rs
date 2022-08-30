use ::std::time::Instant;

use ::log::debug;

use crate::common::{LineWriter, Task, VecWriter};
use crate::ExitStatus;
use crate::observe::mon_args::MonArgs;
use crate::observe::piped_args::PipedArgs;
use crate::observe::sound_notification;

pub async fn piped(args: PipedArgs, writer: &mut impl LineWriter) -> ExitStatus {
    unimplemented!()  //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
