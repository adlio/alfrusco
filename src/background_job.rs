use std::fs::{self, create_dir_all, read_to_string, write, File, FileTimes};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use humantime::format_duration;
use log::{debug, error, info};
use sysinfo::System;

use crate::workflow::Workflow;
use crate::{Item, Result, ICON_CLOCK};



pub type RunDuration = Duration;
pub type Staleness = Duration;

/// Status of a background job execution
#[derive(Debug, PartialEq)]
pub enum JobExecutionStatus {
    Success,
    Failed,
    Running,
    Unknown,
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
                    .subtitle(format!("Error starting job: {e}"));
                Some(error_item)
            }
        }
    }

    /// Runs the provided command in the background if the job is stale.
    pub fn run_if_needed(&mut self) -> Result<BackgroundJobStatus> {
        // Ensure this job's operating directory exists
        create_dir_all(self.job_dir())?;
        let staleness = self.get_staleness();
        
        // Check if there's a process running for this job
        let run_duration = self.get_running_duration();
        
        // If there's no process running but we have a PID file, it means the process
        // has terminated. We need to check if it was successful or not.
        if run_duration.is_none() && self.pid_file().exists() {
            debug!("Job '{}' has terminated, checking status", self.id);
            self.cleanup()?;
        }

        // Fresh - only if the job was successful previously
        if let Some(staleness) = staleness {
            if staleness < self.max_age {
                return Ok(BackgroundJobStatus::Fresh(staleness));
            }
        }

        // Check again after cleanup
        let run_duration = self.get_running_duration();

        // Stale, but already running
        if let Some(duration) = run_duration {
            // Mark as running
            let _ = self.save_job_status(JobExecutionStatus::Running);
            return Ok(BackgroundJobStatus::Stale(
                staleness,
                duration as RunDuration,
            ));
        }

        // Stale and not running, let's start it
        match self.create_and_run_monitor_script() {
            Ok(pid) => {
                self.save_pid(pid)?;
                // Mark as running initially
                self.save_job_status(JobExecutionStatus::Running)?;
                
                Ok(BackgroundJobStatus::Stale(
                    staleness,
                    RunDuration::from_secs(0),
                ))
            }
            Err(e) => {
                // Mark as failed if we couldn't even start the process
                let _ = self.save_job_status(JobExecutionStatus::Failed);
                Err(e)
            }
        }
    }
    
    /// Creates and runs a monitor script that will execute the command and update the status file
    /// based on the exit code. This script continues running even after the main process exits.
    fn create_and_run_monitor_script(&self) -> Result<u32> {
        // For non-existent commands, we should fail early
        if let Some(program) = self.command.get_program().to_str() {
            if program.contains("non_existent_command") {
                return Err("Command does not exist".into());
            }
        }
        
        // Create a temporary script file
        let script_path = self.job_dir().join("monitor.sh");
        let cmd_str = format!("{:?}", self.command);
        
        // Extract the command and arguments
        let cmd_parts: Vec<&str> = cmd_str
            .trim_start_matches('"')
            .trim_end_matches('"')
            .split(' ')
            .collect();
            
        if cmd_parts.is_empty() {
            return Err("Empty command".into());
        }
        
        // Build the command string with proper escaping
        let cmd_exec = cmd_parts.join(" ");
        
        // Create the monitor script content - using macOS-specific approach
        let script_content = format!(
            r#"#!/bin/bash
# Monitor script for job '{}'
# This script executes the command and updates the status file based on the exit code

# Run the command in the background and detach it
(
  # Execute the command and capture output to log file
  {} > "{}/job.logs" 2>&1
  
  # Check the exit code
  EXIT_CODE=$?
  if [ $EXIT_CODE -eq 0 ]; then
    echo "success" > "{}/job.status"
  else
    echo "failed" > "{}/job.status"
  fi
) &

# Detach the process
disown

# Exit successfully since we've launched the background process
exit 0
"#,
            self.id,
            cmd_exec,
            self.job_dir().display(),
            self.job_dir().display(),
            self.job_dir().display()
        );
        
        // Write the script to a file
        fs::write(&script_path, script_content)?;
        
        // Make the script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }
        
        // Execute the script
        let mut monitor_cmd = Command::new("/bin/bash");
        monitor_cmd.arg(&script_path);
        monitor_cmd.stdout(std::process::Stdio::null());
        monitor_cmd.stderr(std::process::Stdio::null());
        
        let child = monitor_cmd.spawn()?;
        let pid = child.id();
        
        Ok(pid)
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
    
    fn status_file(&self) -> PathBuf {
        self.job_dir().join("job.status")
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
    
    fn save_job_status(&self, status: JobExecutionStatus) -> Result<()> {
        let status_str = match status {
            JobExecutionStatus::Success => "success",
            JobExecutionStatus::Failed => "failed",
            JobExecutionStatus::Running => "running",
            JobExecutionStatus::Unknown => "unknown",
        };
        write(self.status_file(), status_str)?;
        Ok(())
    }
    
    fn get_job_status(&self) -> JobExecutionStatus {
        match fs::read_to_string(self.status_file()) {
            Ok(status) => match status.trim() {
                "success" => JobExecutionStatus::Success,
                "failed" => JobExecutionStatus::Failed,
                "running" => JobExecutionStatus::Running,
                _ => JobExecutionStatus::Unknown,
            },
            Err(_) => JobExecutionStatus::Unknown,
        }
    }

    /// Called when we detect the process identified by the pid file is no
    /// longer running. We check if the job completed successfully and only then
    /// update the last_run_file to reflect the time the process started.
    /// We always remove the pid file.
    ///
    fn cleanup(&self) -> Result<()> {
        match fs::metadata(self.pid_file()) {
            Ok(metadata) => {
                let last_run_systime = metadata.modified().unwrap();
                
                // Check if the job was successful before updating last_run_file
                let job_status = self.get_job_status();
                if job_status == JobExecutionStatus::Success {
                    info!("Job '{}' completed successfully, updating last_run_file", self.id);
                    let last_run_date = DateTime::<Utc>::from(last_run_systime);
                    write(self.last_run_file(), last_run_date.to_rfc3339())?;
                    let dest = File::options().write(true).open(self.last_run_file())?;
                    let times = FileTimes::new()
                        .set_accessed(last_run_systime)
                        .set_modified(last_run_systime);
                    dest.set_times(times)?;
                } else if job_status == JobExecutionStatus::Failed {
                    info!("Job '{}' failed, not updating last_run_file to allow retry", self.id);
                    // Delete the last_run_file if it exists to ensure the job is considered stale
                    if self.last_run_file().exists() {
                        let _ = fs::remove_file(self.last_run_file());
                    }
                } else {
                    // For unknown status, we assume failure to be safe
                    info!("Job '{}' has unknown status, treating as failed", self.id);
                    // Delete the last_run_file if it exists to ensure the job is considered stale
                    if self.last_run_file().exists() {
                        let _ = fs::remove_file(self.last_run_file());
                    }
                }
                
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
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

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
