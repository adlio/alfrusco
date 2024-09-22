use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::background_job::BackgroundJob;
use crate::workflow::Workflow;

impl Workflow {
    /// Ensure that a particular command is run at least as often as the
    /// provided max_age value. A background job status item is added to
    /// the response items if the job is stale to inform the user that
    /// work is being done in the background to update results.
    ///
    pub fn run_in_background(&mut self, job_key: &str, max_age: Duration, cmd: Command) {
        let mut job = BackgroundJob::new(self, job_key, max_age, cmd);
        let job_item = job.run();
        if let Some(item) = job_item {
            self.response.prepend_items(vec![item]);
        }
    }

    /// Returns the path to the cache subdirectory where jobs data is held
    pub fn jobs_dir(&self) -> PathBuf {
        self.config.workflow_cache.join("jobs")
    }
}
