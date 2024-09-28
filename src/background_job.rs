use std::fs::{self, create_dir_all, read_to_string, write, File, FileTimes};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use humantime::format_duration;
use log::{debug, error};
use sysinfo::System;

use crate::workflow::Workflow;
use crate::{Item, Result, ICON_CLOCK};

pub type RunDuration = Duration;
pub type Staleness = Duration;

pub(crate) struct BackgroundJob<'a> {
    /// The unique identifier/name for this background job
    id: &'a str,

    /// The maximum time allowed since the job was last run
    /// before it is considered stale and we re-run it.
    max_age: Duration,

    /// The command to run to update the data for this job
    command: Command,

    /// The workflow this job is associated with
    workflow: &'a Workflow,
}

/// BackgroundJobStatus reflects the current state of a requested background
/// task. The task can either be fresh or stale, and if stale, it can either
/// be in the process of running, or known to have failed.
///
#[derive(Debug)]
pub enum BackgroundJobStatus {
    Fresh(Staleness),
    Stale(Option<Staleness>, RunDuration),
}

impl<'a> BackgroundJob<'a> {
    pub fn new(
        workflow: &'a Workflow,
        name: &'a str,
        max_age: Duration,
        command: Command,
    ) -> BackgroundJob<'a> {
        let mut command = command;

        // Ensure that the spawned command gets its own STDOUT, while
        // STDERR is inherited from the parent process.
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::inherit());
        BackgroundJob {
            workflow,
            id: name,
            max_age,
            command,
        }
    }

    pub fn run(&mut self) -> Option<Item> {
        use BackgroundJobStatus::*;

        let status = self.run_if_needed();
        match status {
            Ok(status) => match status {
                Fresh(staleness) => {
                    debug!(
                        "Job '{}' is fresh, last run {}",
                        self.id,
                        format_duration(staleness)
                    );
                    None
                }
                Stale(staleness, duration) => match staleness {
                    Some(staleness) => {
                        debug!(
                            "Job '{}' is stale. Last run {} ago, running for {}",
                            self.id,
                            format_duration(staleness),
                            format_duration(duration),
                        );
                        // Truncate to milliseconds
                        let staleness = Duration::from_millis(staleness.as_millis() as u64);
                        let duration = Duration::from_millis(duration.as_millis() as u64);
                        let stale_item = Item::new(format!("Background Job '{}'", self.id))
                            .subtitle(format!(
                                "Job is stale by {}, running for {}",
                                format_duration(staleness),
                                format_duration(duration)
                            ))
                            .icon(ICON_CLOCK.into())
                            .valid(false);
                        Some(stale_item)
                    }
                    None => {
                        debug!(
                            "Job '{}' has never run before, running for {}",
                            self.id,
                            format_duration(duration)
                        );
                        let stale_item = Item::new(format!("Background Job '{}'", self.id))
                            .subtitle(format!(
                                "Job is stale, running for {}",
                                format_duration(duration)
                            ))
                            .icon(ICON_CLOCK.into())
                            .valid(false);
                        Some(stale_item)
                    }
                },
            },
            Err(e) => {
                error!("Error starting job '{}': {}", self.id, e);
                let error_item = Item::new(format!("Background Job '{}'", self.id))
                    .subtitle(format!("Error starting job: {}", e));
                Some(error_item)
            }
        }
    }

    /// Runs the provided command in the background if the job is stale.
    pub fn run_if_needed(&mut self) -> Result<BackgroundJobStatus> {
        // Ensure this job's operating directory exists
        create_dir_all(self.job_dir())?;
        let staleness = self.get_staleness();

        // Fresh
        if let Some(staleness) = staleness {
            if staleness < self.max_age {
                return Ok(BackgroundJobStatus::Fresh(staleness));
            }
        }

        let run_duration = self.get_running_duration();

        // Stale, but already running
        if let Some(duration) = run_duration {
            return Ok(BackgroundJobStatus::Stale(
                staleness,
                duration as RunDuration,
            ));
        }

        self.cleanup()?;

        // Stale and not running, let's start it
        match self.command.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.save_pid(pid)?;
                Ok(BackgroundJobStatus::Stale(
                    staleness,
                    RunDuration::from_secs(0),
                ))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn job_dir(&self) -> PathBuf {
        self.workflow.jobs_dir().join(self.id)
    }

    fn pid_file(&self) -> PathBuf {
        self.job_dir().join("job.pid")
    }

    fn last_run_file(&self) -> PathBuf {
        self.job_dir().join("job.last_run")
    }

    fn get_pid(&self) -> Result<u32> {
        let pid = read_to_string(self.pid_file())?;
        pid.trim().parse::<u32>().map_err(|e| e.into())
    }

    fn save_pid(&self, pid: u32) -> Result<()> {
        write(self.pid_file(), pid.to_string())?;
        Ok(())
    }

    fn delete_pid_file(&self) -> Result<()> {
        // Check if the file exists before trying to remove it
        if !self.pid_file().exists() {
            return Ok(());
        }
        fs::remove_file(self.pid_file())?;
        Ok(())
    }

    /// Called when we detect the process identified by the pid file is no
    /// longer running. We update the last_run_file to reflect the time the
    /// process started, and remove the pid file.
    ///
    fn cleanup(&self) -> Result<()> {
        match fs::metadata(self.pid_file()) {
            Ok(metadata) => {
                let last_run_systime = metadata.modified().unwrap();
                let last_run_date = DateTime::<Utc>::from(last_run_systime);
                write(self.last_run_file(), last_run_date.to_rfc3339())?;
                let dest = File::options().write(true).open(self.last_run_file())?;
                let times = FileTimes::new()
                    .set_accessed(last_run_systime)
                    .set_modified(last_run_systime);
                dest.set_times(times)?;
                self.delete_pid_file()?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    /// If the specified job is running, this returns the duration since it
    /// started. Otherwise, it returns None.
    ///
    fn get_running_duration(&self) -> Option<Duration> {
        let mut system = System::new_all();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let pid = self.get_pid();
        match pid {
            Ok(pid) => system.process(sysinfo::Pid::from(pid as usize)).map(|p| {
                let start_time = UNIX_EPOCH + Duration::from_secs(p.start_time());
                SystemTime::now()
                    .duration_since(start_time)
                    .unwrap_or_default()
            }),
            Err(_) => None,
        }
    }

    /// If the specified job has successfully started before, this returns the duration
    /// since that event occurred. Otherwise, it returns None. We use the file timestamp
    /// on an empty file to determine the last completion time.
    fn get_staleness(&self) -> Option<Staleness> {
        match fs::metadata(self.last_run_file()) {
            Ok(metadata) => {
                let last_run = metadata.modified().unwrap();
                let duration = SystemTime::now().duration_since(last_run).unwrap();
                Some(duration)
            }
            Err(_) => None,
        }
    }
}
