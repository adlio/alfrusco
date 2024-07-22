use std::path::PathBuf;

use crate::Result;

pub struct Workflow {
    pub name: String,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl Workflow {
    /// Creates a new Workflow instance by reading the environment variables.
    /// This will fail with a an Error::VarError if any of the required
    /// environment variables are not set.
    ///
    /// See https://www.alfredapp.com/help/workflows/script-environment-variables/
    /// for more information on the Alfred workflow environment variables.
    pub fn new() -> Result<Workflow> {
        let name = std::env::var("alfred_workflow_name")?;
        let cache_dir = std::env::var("alfred_workflow_cache")?;
        let data_dir = std::env::var("alfred_workflow_data")?;
        Ok({
            Workflow {
                name,
                data_dir: PathBuf::from(data_dir),
                cache_dir: PathBuf::from(cache_dir),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_workflow() {
        let _workflow = Workflow::new().unwrap();
    }
}
