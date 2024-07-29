use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;

use crate::Result;
use crate::Workflow;

/// BackgroundTaskStatus reflects the current state of a requested background
/// task. The task can either be fresh or stale, and if stale, it can either
/// be in the process of running, or known to have failed.
///
pub enum BackgroundTaskStatus {
    Fresh(Duration),
    // TODO: This should wrap a struct to clarify the meaning of the durations
    StaleAndRunning(Duration, Duration),
    StaleAndFailed(Duration, String),
}

pub struct BGTask {
    pub id: String,
    pub max_age: Duration,
    pub ttl: Duration,
}

pub struct BGTaskExecutionResult {
    pub last_completion: chrono::DateTime<chrono::Utc>,
}

impl Workflow {
    pub fn run_in_background(
        &self,
        job_name: &str,
        max_age: Duration,
        mut cmd: Command,
    ) -> Result<BackgroundTaskStatus> {
        // TODO This should be a let staleness = ...
        match self.get_duration_since_last_completion(job_name) {
            Some(duration) => {
                if duration < max_age {
                    return Ok(BackgroundTaskStatus::Fresh(duration));
                }
            }
            None => {}
        }

        match self.get_pid_running_duration(job_name) {
            Some(duration) => {
                return Ok(BackgroundTaskStatus::StaleAndRunning(duration, duration));
            }
            None => {
                // TODO Fix this
                Ok(BackgroundTaskStatus::Fresh(self.max_age))
            }
        }
    }

    fn is_running(&self, job_name: &str) -> bool {
        match self.get_pid(job_name) {
            Ok(pid) => {
                let mut system = System::new_all();
                system.refresh_processes();
                system.process(sysinfo::Pid::from(pid as usize)).is_some()
            }
            Err(_) => false,
        }
    }

    fn get_pid_running_duration(&self, job_name: &str) -> Option<Duration> {
        match self.get_pid(job_name) {
            Ok(pid) => {
                let mut system = System::new_all();
                system.refresh_processes();
                system.process(sysinfo::Pid::from(pid as usize)).map(|p| {
                    let start_time = UNIX_EPOCH + Duration::from_secs(p.start_time());
                    SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default()
                })
            }
            Err(_) => None,
        }
    }

    fn get_pid(&self, job_name: &str) -> Result<u32> {
        let pid_file = self.cache_dir.join(format!("{}.pid", job_name));
        let pid = fs::read_to_string(pid_file)?;
        pid.trim().parse::<u32>().map_err(|e| e.into())
    }

    fn save_pid(&self, job_name: &str, pid: u32) -> Result<()> {
        let pid_file = self.cache_dir.join(format!("{}.pid", job_name));
        fs::write(pid_file, pid.to_string())?;
        Ok(())
    }

    /// If the specified job has successfully completed before, this returns the duration
    /// since that event occurred. Otherwise, it returns None. We use the file timestamp
    /// on an empty file to determine the last completion time.
    fn get_duration_since_last_completion(&self, job_name: &str) -> Option<Duration> {
        let last_completion_file = self.cache_dir.join(format!("{}.last", job_name));
        match fs::metadata(last_completion_file) {
            Ok(metadata) => {
                let last_completion = metadata.modified().unwrap();
                let duration = SystemTime::now().duration_since(last_completion).unwrap();
                Some(duration)
            }
            Err(_) => None,
        }
    }

    fn update_last_completion(&self, job_name: &str) -> Result<()> {
        let last_completion_file = self.cache_dir.join(format!("{}.last", job_name));
        // We write the time in the local time zone. This is mainly for human legibility.
        // We actually use the modified timestamp of the file to determine the age
        fs::write(last_completion_file, chrono::Local::now().to_rfc3339())?;
        Ok(())
    }
}
