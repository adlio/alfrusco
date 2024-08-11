use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::background_job::{BackgroundJob, BackgroundJobStatus};
use crate::Result;
use crate::Workflow;

impl Workflow {
    pub fn run_in_background(
        &self,
        job_name: &str,
        max_age: Duration,
        cmd: Command,
    ) -> Result<BackgroundJobStatus> {
        let job_path = self.jobs_dir().join(job_name);
        std::fs::create_dir_all(&job_path)?;

        let mut job = BackgroundJob::new(job_path, max_age, cmd);
        job.run_if_needed()
    }

    /// Returns the path to the cache subdirectory where jobs data is held
    pub fn jobs_dir(&self) -> PathBuf {
        self.cache_dir().join("jobs")
    }
}
