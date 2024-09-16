use crate::Result;
use std::path::PathBuf;
use std::{env, fs};

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
const VAR_WORKFLOW_KEYWORD: &str = "alfred_workflow_keyword";
const VAR_DEBUG: &str = "alfred_debug";

/// WorkflowConfig holds the configuration values for the current workflow.
/// In a real-world scenario, these values are read from environment variables.
/// The from_env() constructor is the primary way to create a WorkflowConfig.
///
/// See https://www.alfredapp.com/help/workflows/script-environment-variables/
pub struct WorkflowConfig {
    pub preferences: String,
    pub preferences_localhash: String,
    pub theme: Option<String>,
    pub theme_background: Option<String>,
    pub theme_selection_background: Option<String>,
    pub theme_subtext: Option<String>,
    pub version: String,
    pub version_build: String,
    pub workflow_bundleid: String,
    pub workflow_cache: PathBuf,
    pub workflow_data: PathBuf,
    pub workflow_name: String,
    pub workflow_description: Option<String>,
    pub workflow_version: String,
    pub workflow_uid: String,
    pub workflow_keyword: Option<String>,
    pub debug: bool,
}

impl WorkflowConfig {
    pub fn from_env() -> Result<Self> {
        let debug = env::var(VAR_DEBUG).unwrap_or_default();
        let debug = debug == "1" || debug.to_lowercase() == "true";

        let config = WorkflowConfig {
            preferences: env::var(VAR_PREFERENCES).ok().unwrap_or_default(),
            preferences_localhash: env::var(VAR_PREFERENCES_LOCALHASH).ok().unwrap_or_default(),
            theme: env::var(VAR_THEME).ok(),
            theme_background: env::var(VAR_THEME_BACKGROUND).ok(),
            theme_selection_background: env::var(VAR_THEME_SELECTION_BACKGROUND).ok(),
            theme_subtext: env::var(VAR_THEME_SUBTEXT).ok(),
            version: env::var(VAR_VERSION).ok().unwrap_or_default(),
            version_build: env::var(VAR_VERSION_BUILD).ok().unwrap_or_default(),
            workflow_bundleid: env::var(VAR_WORKFLOW_BUNDLEID).ok().unwrap_or_default(),
            workflow_cache: env::var(VAR_WORKFLOW_CACHE).ok().unwrap_or_default().into(),
            workflow_data: env::var(VAR_WORKFLOW_DATA).ok().unwrap_or_default().into(),
            workflow_name: env::var(VAR_WORKFLOW_NAME).ok().unwrap_or_default(),
            workflow_description: env::var(VAR_WORKFLOW_DESCRIPTION).ok(),
            workflow_version: env::var(VAR_WORKFLOW_VERSION).ok().unwrap_or_default(),
            workflow_uid: env::var(VAR_WORKFLOW_UID).ok().unwrap_or_default(),
            workflow_keyword: env::var(VAR_WORKFLOW_KEYWORD).ok(),
            debug,
        };

        std::fs::create_dir_all(&config.workflow_cache)?;
        std::fs::create_dir_all(&config.workflow_data)?;

        Ok(config)
    }

    pub fn for_testing() -> Result<Self> {
        let current_dir = env::current_dir()?;
        let test_workflow = current_dir.join("test_workflow");

        let workflow_data = test_workflow.join("workflow_data");
        fs::create_dir_all(&workflow_data)?;

        let workflow_cache = test_workflow.join("workflow_cache");
        fs::create_dir_all(&workflow_cache)?;

        Ok(WorkflowConfig {
            preferences: "/Users/Crayons/Dropbox/Alfred/Alfred.alfredpreferences".to_string(),
            preferences_localhash: "adbd4f66bc3ae8493832af61a41ee609b20d8705".to_string(),
            theme: Some("alfred.theme.yosemite".to_string()),
            theme_background: Some("rgba(255,255,255,0.98)".to_string()),
            theme_selection_background: Some("rgba(255,255,255,0.98)".to_string()),
            theme_subtext: Some("3".to_string()),
            version: "5.0".to_string(),
            version_build: "2058".to_string(),
            workflow_bundleid: "com.alfredapp.googlesuggest".to_string(),
            workflow_cache,
            workflow_data,
            workflow_name: "Test Workflow".to_string(),
            workflow_description: Some(
                "The description of the workflow we use for testing".to_string(),
            ),
            workflow_version: "1.7".to_string(),
            workflow_uid: "user.workflow.B0AC54EC-601C-479A-9428-01F9FD732959".to_string(),
            workflow_keyword: None,
            debug: true,
        })
    }
}
