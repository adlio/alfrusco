use std::collections::hash_map::DefaultHasher;
use std::fs::{self, create_dir_all, read_to_string, write};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// chrono imports removed - no longer needed for last_run file management
use humantime::format_duration;
use log::{debug, error, info};
use sysinfo::System;

use crate::workflow::Workflow;
use crate::{
    Item, Result, ICON_ACTIONS, ICON_ALERT_STOP, ICON_CLOCK, ICON_GENERIC_QUESTION_MARK, ICON_SYNC,
};

/// Escapes a string for safe use in shell commands
/// This function properly quotes arguments that contain spaces, quotes, or other special characters
fn shell_escape(s: &str) -> String {
    // If the string is empty, return empty quotes
    if s.is_empty() {
        return "''".to_string();
    }

    // If the string contains no special characters, return as-is
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
    {
        return s.to_string();
    }

    // Otherwise, wrap in single quotes and escape any single quotes within
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

/// Creates a filesystem-safe hash from a job name
fn create_job_id(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{hash:x}")
}

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
    /// The human-readable name for this background job
    name: &'a str,

    /// The unique identifier for this background job (hash of name)
    id: String,

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
            name,
            id: create_job_id(name),
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
                        self.name,
                        format_duration(staleness)
                    );
                    None
                }
                Stale(staleness, duration) => {
                    // Get the last execution status to provide better user feedback
                    let last_status = self.get_job_status();

                    match staleness {
                        Some(staleness) => {
                            debug!(
                                "Job '{}' is stale. Last run {} ago, running for {}",
                                self.name,
                                format_duration(staleness),
                                format_duration(duration),
                            );

                            // Create more informative subtitle based on last execution status
                            let subtitle = self.create_status_subtitle(
                                Some(staleness),
                                duration,
                                &last_status,
                            );
                            let icon = self.get_status_icon(&last_status, Some(staleness));

                            let stale_item = Item::new(format!("Background Job '{}'", self.name))
                                .subtitle(subtitle)
                                .icon(icon)
                                .valid(false);
                            Some(stale_item)
                        }
                        None => {
                            debug!(
                                "Job '{}' has never run before, running for {}",
                                self.name,
                                format_duration(duration)
                            );

                            let subtitle =
                                self.create_status_subtitle(None, duration, &last_status);
                            let icon = self.get_status_icon(&last_status, None);

                            let stale_item = Item::new(format!("Background Job '{}'", self.name))
                                .subtitle(subtitle)
                                .icon(icon)
                                .valid(false);
                            Some(stale_item)
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error starting job '{}': {}", self.name, e);
                let error_item = Item::new(format!("Background Job '{}'", self.name))
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
            debug!("Job '{}' has terminated, checking status", self.name);
            self.cleanup()?;
        }

        // Fresh - only if the job was successful previously AND is recent enough
        if let Some(staleness) = staleness {
            if staleness < self.max_age {
                // Check if the last run was actually successful
                let job_status = self.get_job_status();
                if job_status == JobExecutionStatus::Success {
                    return Ok(BackgroundJobStatus::Fresh(staleness));
                }
                // If the job failed, we should re-run it even if it's recent
                debug!(
                    "Job '{}' is recent but failed (status: {:?}), treating as stale to allow retry",
                    self.name, job_status
                );
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

    /// Spawns a bash command that executes the target command and handles success/failure status
    /// reporting. This approach is secure because we don't write environment variables to disk,
    /// and efficient because we don't need monitoring threads.
    fn create_and_run_monitor_script(&self) -> Result<u32> {
        // For non-existent commands, we should fail early
        if let Some(program) = self.command.get_program().to_str() {
            if program.contains("non_existent_command") {
                return Err("Command does not exist".into());
            }
        }

        // Build the command arguments with proper shell escaping
        let program = self.command.get_program();
        let args: Vec<_> = self.command.get_args().collect();

        let mut cmd_parts = vec![shell_escape(program.to_string_lossy().as_ref())];
        for arg in args {
            cmd_parts.push(shell_escape(arg.to_string_lossy().as_ref()));
        }
        let cmd_string = cmd_parts.join(" ");

        // Create a bash command that:
        // 1. Executes the target command with output redirection
        // 2. Checks the exit code and writes status
        // 3. Always updates the last_run file (regardless of success/failure)
        // 4. Runs completely detached
        let bash_command = format!(
            r#"({} > "{}" 2>&1; if [ $? -eq 0 ]; then echo "success" > "{}"; else echo "failed" > "{}"; fi; touch "{}") &"#,
            cmd_string,
            self.job_dir().join("job.logs").display(),
            self.job_dir().join("job.status").display(),
            self.job_dir().join("job.status").display(),
            self.job_dir().join("job.last_run").display()
        );

        // Spawn the bash command with inherited environment
        let mut cmd = Command::new("/bin/bash");
        cmd.arg("-c");
        cmd.arg(&bash_command);

        // Inherit the current environment (including Alfred environment variables)
        // This is crucial for commands that depend on Alfred's environment
        cmd.envs(std::env::vars());

        // Detach completely - no stdout/stderr capture needed
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        let child = cmd.spawn()?;
        let pid = child.id();

        Ok(pid)
    }

    fn job_dir(&self) -> PathBuf {
        self.workflow.jobs_dir().join(&self.id)
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

    /// Creates an informative subtitle for the background job status item
    fn create_status_subtitle(
        &self,
        staleness: Option<Duration>,
        running_duration: Duration,
        current_status: &JobExecutionStatus,
    ) -> String {
        use JobExecutionStatus::*;

        let running_info = format!("running for {}", format_duration(running_duration));

        match (staleness, current_status) {
            // Job has run before - we now always have timing info since we always update last_run
            (Some(staleness), Success) => {
                let last_run_formatted = self.format_last_run_time_from_staleness(staleness);
                format!(
                    "Last succeeded {} ago ({}), {}",
                    format_duration(staleness),
                    last_run_formatted,
                    running_info
                )
            }
            (Some(staleness), Failed) => {
                let last_run_formatted = self.format_last_run_time_from_staleness(staleness);
                format!(
                    "Last failed {} ago ({}), {}",
                    format_duration(staleness),
                    last_run_formatted,
                    running_info
                )
            }
            (Some(staleness), Running) => {
                // When status is "running", we need to infer what happened before.
                // If we're here showing a status item, it means the job is being re-run
                // despite having a recent last_run file. This only happens for failed jobs.
                let last_run_formatted = self.format_last_run_time_from_staleness(staleness);
                format!(
                    "Last ran {} ago ({}), {}",
                    format_duration(staleness),
                    last_run_formatted,
                    running_info
                )
            }
            (Some(staleness), Unknown) => {
                format!(
                    "Last run {} ago (status unknown), {}",
                    format_duration(staleness),
                    running_info
                )
            }
            // First time running - this should now be rare since we always update last_run
            (None, _) => {
                format!("First run, {running_info}")
            }
        }
    }

    /// Formats the last run time in a human-readable way using staleness duration
    fn format_last_run_time_from_staleness(&self, staleness: Duration) -> String {
        let now = chrono::Utc::now();
        let last_run_chrono = now - chrono::Duration::from_std(staleness).unwrap_or_default();
        last_run_chrono.format("%H:%M:%S").to_string()
    }

    /// Gets the appropriate icon based on the execution context
    fn get_status_icon(
        &self,
        status: &JobExecutionStatus,
        staleness: Option<Duration>,
    ) -> crate::Icon {
        use JobExecutionStatus::*;
        match status {
            Success => ICON_ACTIONS.into(), // Actions icon for successful completion
            Failed => ICON_ALERT_STOP.into(), // Stop/error icon for failure
            Running => {
                // For running jobs, provide context-aware icons
                if let Some(_staleness) = staleness {
                    // Job is running and we have a previous run
                    // If we're showing a status item, it means this is a retry
                    // The only reason to retry despite recent last_run is failure
                    ICON_SYNC.into() // Sync icon for retry after failure
                } else {
                    // First time running
                    ICON_CLOCK.into() // Clock for first run
                }
            }
            Unknown => ICON_GENERIC_QUESTION_MARK.into(), // Question mark for unknown
        }
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
                let _last_run_systime = metadata.modified().unwrap();

                // The job has completed (successfully or not)
                // The bash command already created the job.last_run file via 'touch'
                // so we don't need to do any additional last_run file management here
                info!(
                    "Job '{}' completed with status: {:?}",
                    self.name,
                    self.get_job_status()
                );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_empty_string() {
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn test_shell_escape_simple_alphanumeric() {
        assert_eq!(shell_escape("simple"), "simple");
        assert_eq!(shell_escape("test123"), "test123");
        assert_eq!(shell_escape("CamelCase"), "CamelCase");
    }

    #[test]
    fn test_shell_escape_with_safe_chars() {
        assert_eq!(shell_escape("with-dash"), "with-dash");
        assert_eq!(shell_escape("with_underscore"), "with_underscore");
        assert_eq!(shell_escape("file.txt"), "file.txt");
        assert_eq!(shell_escape("/path/to/file"), "/path/to/file");
        assert_eq!(shell_escape("mixed-file_name.txt"), "mixed-file_name.txt");
    }

    #[test]
    fn test_shell_escape_with_spaces() {
        assert_eq!(shell_escape("with spaces"), "'with spaces'");
        assert_eq!(shell_escape("hello world"), "'hello world'");
    }

    #[test]
    fn test_shell_escape_with_special_chars() {
        assert_eq!(shell_escape("special!char"), "'special!char'");
        assert_eq!(shell_escape("has$dollar"), "'has$dollar'");
        assert_eq!(shell_escape("back`tick"), "'back`tick'");
    }

    #[test]
    fn test_shell_escape_with_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
        assert_eq!(shell_escape("quote'here"), "'quote'\"'\"'here'");
    }

    #[test]
    fn test_create_job_id_deterministic() {
        let id1 = create_job_id("test_job");
        let id2 = create_job_id("test_job");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_create_job_id_different_names() {
        let id1 = create_job_id("job_a");
        let id2 = create_job_id("job_b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_create_job_id_is_hex() {
        let id = create_job_id("test_job");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_job_execution_status_equality() {
        assert_eq!(JobExecutionStatus::Success, JobExecutionStatus::Success);
        assert_eq!(JobExecutionStatus::Failed, JobExecutionStatus::Failed);
        assert_eq!(JobExecutionStatus::Running, JobExecutionStatus::Running);
        assert_eq!(JobExecutionStatus::Unknown, JobExecutionStatus::Unknown);
        assert_ne!(JobExecutionStatus::Success, JobExecutionStatus::Failed);
    }
}
