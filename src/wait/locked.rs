use crate::wait::LockedArgs;

#[allow(unreachable_code)] //TODO @mverleg: TEMPORARY! REMOVE THIS!
pub fn locked(args: LockedArgs) -> Result<(), String> {
    if args.show {
        unimplemented!();
        return Ok(());
    }
    if args.unlock {
        unimplemented!();
        return Ok(());
    }
    let task = args.cmd.into_task();
    let status = task.execute_sync(true);
    if status.is_err() {
        return Err(format!("failed with status {}", status.code()));
    }
    //TODO @mverleg:
    Ok(())
}
