use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::time::Duration;

use crate::Result;
use crate::Workflow;

/// BackgroundTaskStatus reflects the current state of a requested background
/// task. The task can either be fresh or stale, and if stale, it can either
/// be in the process of running, or known to have failed.
///
pub enum BackgroundTaskStatus {
    Fresh(Duration),
    StaleAndRunning(Duration, Duration),
    StaleAndFailed(Duration, String),
}

pub struct BGTask {
    pub id: String,
    pub max_age: Duration,
    pub ttl: Duration,
}

impl Workflow {
    pub fn run_in_background(&self, job_name: &str, mut cmd: Command) -> io::Result<Child> {
        if self.is_running(job_name) {
            let pid = self.get_pid(job_name)?;
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                ErrJobExists {
                    job_name: job_name.to_string(),
                    pid,
                },
            ));
        }

        // Set the process group ID to prevent the process from being killed when the parent is
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let child = cmd.spawn()?;

        self.save_pid(job_name, child.id())?;

        Ok(child)
    }

    fn is_running(&self, job_name: &str) -> bool {
        self.get_pid(job_name).is_ok()
    }

    fn get_pid(&self, job_name: &str) -> Result<u32> {
        let pid_file = self.cache_dir.join(format!("{}.pid", job_name));
        let pid_str = fs::read_to_string(pid_file)?;
        pid_str.trim().parse::<u32>().map_err(|e| e.into())
    }

    fn save_pid(&self, job_name: &str, pid: u32) -> Result<()> {
        let pid_file = self.cache_dir.join(format!("{}.pid", job_name));
        fs::write(pid_file, pid.to_string())?;
        Ok(())
    }
}
