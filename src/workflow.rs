use std::path::PathBuf;

use crate::{Item, Result};

const VAR_WORKFLOW_NAME: &str = "alfred_workflow_name";
const VAR_WORKFLOW_BUNDLE_ID: &str = "alfred_workflow_bundleid";
const VAR_WORKFLOW_VERSION: &str = "alfred_workflow_version";
const VAR_WORKFLOW_UID: &str = "alfred_workflow_uid";
const VAR_WORKFLOW_CACHE_DIR: &str = "alfred_workflow_cache";
const VAR_WORKFLOW_DATA_DIR: &str = "alfred_workflow_data";
const VAR_DEBUG: &str = "alfred_debug";
const VAR_THEME: &str = "alfred_theme";
const VAR_THEME_BG: &str = "alfred_theme_background";
const VAR_THEME_SELECTION_BG: &str = "alfred_theme_selection_background";
const VAR_VERSION: &str = "alfred_version";
const VAR_VERSION_BUILD: &str = "alfred_version_build";
const VAR_PREFERENCES: &str = "alfred_preferences";
const VAR_PREFERENCES_LOCAL_HASH: &str = "alfred_preferences_localhash";

pub struct Workflow {
    pub name: String,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,

    pub items: Vec<Item>,
}

impl Workflow {
    /// Creates a new Workflow instance by reading the environment variables.
    /// This will fail with a an Error::VarError if any of the required
    /// environment variables are not set.
    ///
    /// See https://www.alfredapp.com/help/workflows/script-environment-variables/
    /// for more information on the Alfred workflow environment variables.
    pub fn new() -> Result<Workflow> {
        let name = std::env::var(VAR_WORKFLOW_NAME)?;
        let cache_dir = std::env::var(VAR_WORKFLOW_CACHE_DIR)?;
        let data_dir = std::env::var(VAR_WORKFLOW_DATA_DIR)?;

        Ok({
            Workflow {
                name,
                data_dir: PathBuf::from(data_dir),
                cache_dir: PathBuf::from(cache_dir),
                items: Vec::new(),
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
