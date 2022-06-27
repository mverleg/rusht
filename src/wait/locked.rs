use crate::wait::LockedArgs;

pub fn locked(args: LockedArgs) -> Result<(), String> {
    if args.show {
        unimplemented!();
        return Ok(())
    }
    if args.unlock {
        unimplemented!();
        return Ok(())
    }
    let task = args.cmd.into_task();
    let status = task.execute(false);
    if !status.success() {
        return Err(format!("failed with status {}", status.code().unwrap_or(0)));
    }
    //TODO @mverleg:
    Ok(())
}
