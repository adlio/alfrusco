Aduse std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sysinfo::System;

use crate::Result;

pub type RunDuration = Duration;
pub type CacheAge = Duration;

pub struct BackgroundJob {
    pub pid_file: PathBuf,
    pub last_completion_file: PathBuf,
    pub max_age: Duration,
    pub command: Command,
}

/// BackgroundJobStatus reflects the current state of a requested background
/// task. The task can either be fresh or stale, and if stale, it can either
/// be in the process of running, or known to have failed.
///
pub enum BackgroundJobStatus {
    Fresh(CacheAge),
    StaleAndRunning(Option<CacheAge>, RunDuration),
    StaleAndFailed(Option<CacheAge>, String),
}

impl BackgroundJob {
    pub fn new(dir: PathBuf, max_age: Duration, command: Command) -> BackgroundJob {
        let pid_file = dir.join("job.pid");
        let last_completion_file = dir.join("job.last_completion");

        BackgroundJob {
            pid_file,
            last_completion_file,
            max_age,
            command,
        }
    }

    /// Runs the provided command in the background if the job is stale.
    pub fn run_if_needed(&mut self) -> Result<BackgroundJobStatus> {
        let cache_age = self.get_cache_age();

        // Fresh
        if let Some(cache_age) = self.get_cache_age() {
            if cache_age < self.max_age {
                return Ok(BackgroundJobStatus::Fresh(cache_age));
            }
        }

        // Stale, but already running
        if let Some(duration) = self.get_pid_running_duration() {
            return Ok(BackgroundJobStatus::StaleAndRunning(
                cache_age,
                duration as RunDuration,
            ));
        }

        // Stale and not running, let's start it
        match &self.command.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.save_pid(pid)?;
                Ok(BackgroundJobStatus::StaleAndRunning(
                    cache_age,
                    RunDuration::from_secs(0),
                ))
            }
            Err(e) => Ok(BackgroundJobStatus::StaleAndFailed(
                cache_age,
                e.to_string(),
            )),
        }
    }

    fn get_pid_running_duration(&self) -> Option<Duration> {
        match self.get_pid() {
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

    fn get_pid(&self) -> crate::Result<u32> {
        let pid = fs::read_to_string(&self.pid_file)?;
        pid.trim().parse::<u32>().map_err(|e| e.into())
    }

    fn save_pid(&self, pid: u32) -> crate::Result<()> {
        fs::write(&self.pid_file, pid.to_string())?;
        Ok(())
    }

    /// If the specified job has successfully completed before, this returns the duration
    /// since that event occurred. Otherwise, it returns None. We use the file timestamp
    /// on an empty file to determine the last completion time.
    fn get_cache_age(&self) -> Option<CacheAge> {
        match fs::metadata(&self.last_completion_file) {
            Ok(metadata) => {
                let last_completion = metadata.modified().unwrap();
                let duration = SystemTime::now().duration_since(last_completion).unwrap();
                Some(duration)
            }
            Err(_) => None,
        }
    }

    fn update_last_completion(&self) -> crate::Result<()> {
        // We write the time in the local time zone. This is mainly for human legibility.
        // We actually use the modified timestamp of the file to determine the age
        fs::write(
            &self.last_completion_file,
            chrono::Local::now().to_rfc3339(),
        )?;
        Ok(())
    }
}
