use std::process::{Child, Command};

#[cfg(unix)]
pub(crate) fn configure(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
pub(crate) fn configure(_command: &mut Command) {}

#[cfg(unix)]
pub(crate) fn terminate(child: &Child) {
    let pid = rustix::process::Pid::from_child(child);
    let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
}

#[cfg(not(unix))]
pub(crate) fn terminate(_child: &Child) {}

pub(crate) fn stop(child: &mut Child) {
    terminate(child);
    let _ = child.kill();
    let _ = child.wait();
}
