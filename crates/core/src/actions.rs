use anyhow::{Result, anyhow};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub fn sigstop(pid: i32) -> Result<()> {
    kill(Pid::from_raw(pid), Signal::SIGSTOP).map_err(|e| anyhow!(e))?;
    Ok(())
}

pub fn sigcont(pid: i32) -> Result<()> {
    kill(Pid::from_raw(pid), Signal::SIGCONT).map_err(|e| anyhow!(e))?;
    Ok(())
}

pub fn sigterm(pid: i32) -> Result<()> {
    kill(Pid::from_raw(pid), Signal::SIGTERM).map_err(|e| anyhow!(e))?;
    Ok(())
}

pub fn sigkill(pid: i32) -> Result<()> {
    kill(Pid::from_raw(pid), Signal::SIGKILL).map_err(|e| anyhow!(e))?;
    Ok(())
}
