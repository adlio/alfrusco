use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs};

use crate::error::WorkflowError;
use crate::{filter_and_sort_items, handle, Item, Response, Result};

const VAR_PREFERENCES: &str = "alfred_preferences";
const VAR_PREFERENCES_LOCALHASH: &str = "alfred_preferences_localhash";
const VAR_THEME: &str = "alfred_theme";
const VAR_THEME_BACKGROUND: &str = "alfred_theme_background";
const VAR_THEME_SELECTION_BACKGROUND: &str = "alfred_theme_selection_background";
const VAR_THEME_SUBTEXT: &str = "alfred_theme_subtext";
const VAR_VERSION: &str = "alfred_version";
const VAR_VERSION_BUILD: &str = "alfred_version_build";
const VAR_WORKFLOW_BUNDLEID: &str = "alfred_workflow_bundleid";
const VAR_WORKFLOW_CACHE: &str = "alfred_workflow_cache";
const VAR_WORKFLOW_DATA: &str = "alfred_workflow_data";
const VAR_WORKFLOW_NAME: &str = "alfred_workflow_name";
const VAR_WORKFLOW_DESCRIPTION: &str = "alfred_workflow_description";
const VAR_WORKFLOW_UID: &str = "alfred_workflow_uid";
const VAR_WORKFLOW_VERSION: &str = "alfred_workflow_version";
const VAR_DEBUG: &str = "alfred_debug";
const VAR_KEYWORD: &str = "alfred_keyword";

/// WorkflowConfig holds the configuration values for the current workflow.
/// In a real-world scenario, these values are read from environment variables.
/// The from_env() constructor is the primary way to create a WorkflowConfig.
///
/// See https://www.alfredapp.com/help/workflows/script-environment-variables/
pub struct Workflow {
    pub writer: Box<dyn Write>,

    pub preferences: String,
    pub preferences_localhash: String,
    pub theme: String,
    pub theme_background: String,
    pub theme_selection_background: String,
    pub theme_subtext: String,
    pub version: String,
    pub version_build: String,
    pub workflow_bundleid: String,
    pub workflow_cache: PathBuf,
    pub workflow_data: PathBuf,
    pub workflow_name: String,
    pub workflow_description: String,
    pub workflow_version: String,
    pub workflow_uid: String,
    pub debug: bool,

    pub keyword: Option<String>,
    pub sort_and_filter_results: bool,

    pub response: Response,
}

impl Workflow {
    pub fn from_env() -> Result<Self> {
        let debug = env::var(VAR_DEBUG).unwrap_or_default();
        let debug = debug == "1" || debug.to_lowercase() == "true";

        let keyword = env::var(VAR_KEYWORD).ok();
        log::error!("Keyword: {:?}", keyword);

        let config = Workflow {
            writer: Box::new(std::io::stdout()),

            preferences: env::var(VAR_PREFERENCES).ok().unwrap_or_default(),
            preferences_localhash: env::var(VAR_PREFERENCES_LOCALHASH).ok().unwrap_or_default(),
            theme: env::var(VAR_THEME).ok().unwrap_or_default(),
            theme_background: env::var(VAR_THEME_BACKGROUND).ok().unwrap_or_default(),
            theme_selection_background: env::var(VAR_THEME_SELECTION_BACKGROUND)
                .ok()
                .unwrap_or_default(),
            theme_subtext: env::var(VAR_THEME_SUBTEXT).ok().unwrap_or_default(),
            version: env::var(VAR_VERSION).ok().unwrap_or_default(),
            version_build: env::var(VAR_VERSION_BUILD).ok().unwrap_or_default(),
            workflow_bundleid: env::var(VAR_WORKFLOW_BUNDLEID).ok().unwrap_or_default(),
            workflow_cache: env::var(VAR_WORKFLOW_CACHE).ok().unwrap_or_default().into(),
            workflow_data: env::var(VAR_WORKFLOW_DATA).ok().unwrap_or_default().into(),
            workflow_name: env::var(VAR_WORKFLOW_NAME).ok().unwrap_or_default(),
            workflow_description: env::var(VAR_WORKFLOW_DESCRIPTION).ok().unwrap_or_default(),
            workflow_version: env::var(VAR_WORKFLOW_VERSION).ok().unwrap_or_default(),
            workflow_uid: env::var(VAR_WORKFLOW_UID).ok().unwrap_or_default(),
            debug,
            keyword,
            sort_and_filter_results: false,
            response: Response::new(),
        };

        std::fs::create_dir_all(&config.workflow_cache)?;
        std::fs::create_dir_all(&config.workflow_data)?;

        Ok(config)
    }

    pub fn for_testing() -> Result<Workflow> {
        let current_dir = env::current_dir()?;
        let test_workflow = current_dir.join("test_workflow");

        let workflow_data = test_workflow.join("workflow_data");
        fs::create_dir_all(&workflow_data)?;

        let workflow_cache = test_workflow.join("workflow_cache");
        fs::create_dir_all(&workflow_cache)?;

        Ok(Workflow {
            // TODO Make this a buffer to ease testing?
            writer: Box::new(std::io::stdout()),

            preferences: "/Users/Crayons/Dropbox/Alfred/Alfred.alfredpreferences".to_string(),
            preferences_localhash: "adbd4f66bc3ae8493832af61a41ee609b20d8705".to_string(),
            theme: "alfred.theme.yosemite".to_string(),
            theme_background: "rgba(255,255,255,0.98)".to_string(),
            theme_selection_background: "rgba(255,255,255,0.98)".to_string(),
            theme_subtext: "3".to_string(),
            version: "5.0".to_string(),
            version_build: "2058".to_string(),
            workflow_bundleid: "com.alfredapp.googlesuggest".to_string(),
            workflow_cache,
            workflow_data,
            workflow_name: "Test Workflow".to_string(),
            workflow_description: "The description of the workflow we use for testing".to_string(),
            workflow_version: "1.7".to_string(),
            workflow_uid: "user.workflow.B0AC54EC-601C-479A-9428-01F9FD732959".to_string(),
            debug: true,
            keyword: Some("search-keyword".to_string()),
            sort_and_filter_results: false,
            response: Response::new(),
        })
    }

    /// Sets the filter keyword for the workflow and enables sorting and filtering of results.
    ///
    /// This function performs the following actions:
    /// 1. Sets the `keyword` field of the workflow to the provided keyword.
    /// 2. Schedules a rerun of the workflow after a 500ms delay.
    /// 3. Enables sorting and filtering of results.
    ///
    /// # Arguments
    ///
    /// * `keyword` - A String that will be used as the new filter keyword.
    pub fn set_filter_keyword(&mut self, keyword: String) {
        self.keyword = Some(keyword);
        self.response.rerun(Duration::from_millis(500));
        self.sort_and_filter_results = true;
    }

    /// Runs
    pub fn run(&mut self, f: impl FnOnce(&mut Workflow) -> std::result::Result<(), WorkflowError>) {
        // If the response includes alfrusco clipboard instructions, handle them
        // first
        handle();

        let result = f(self);
        if let Err(err) = result {
            let error_item = Item::new(format!("Error: {}", err))
                .subtitle("Check the logs for more information.");
            self.response.prepend_items(vec![error_item]);
        }

        if self.sort_and_filter_results {
            if let Some(keyword) = self.keyword.clone() {
                // TODO Don't clone the items
                self.response.items = filter_and_sort_items(self.response.items.clone(), keyword)
            }
        }

        match self.response.write(&mut self.writer) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error writing response: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_run_success() {
        let mut wf = Workflow::for_testing().unwrap();
        wf.run(|wf| {
            wf.response
                .append_items(vec![Item::new("Test Item").subtitle("Test Subtitle")]);
            Ok(())
        });
        assert_eq!(wf.response.items.len(), 1);
        assert_eq!(wf.response.items[0].title, "Test Item");
    }

    #[test]
    fn test_sync_run_error() {
        let mut wf = Workflow::for_testing().unwrap();
        wf.run(|_wf| Err::<(), _>("Test error".into()));
        assert_eq!(wf.response.items.len(), 1);
        assert!(wf.response.items[0].title.contains("Error"));
    }
}
