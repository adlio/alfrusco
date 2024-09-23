use std::env;
use std::path::PathBuf;

use crate::Result;

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
///
/// In a real-world scenario, these values are read from environment variables.
/// The from_env() constructor is the primary way to create a WorkflowConfig.
///
/// See https://www.alfredapp.com/help/workflows/script-environment-variables/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WorkflowConfig {
    pub workflow_bundleid: String,
    pub workflow_cache: PathBuf,
    pub workflow_data: PathBuf,
    pub version: String,
    pub version_build: String,
    pub workflow_name: String,
    pub workflow_version: String,

    pub preferences: Option<String>,
    pub preferences_localhash: Option<String>,
    pub theme: Option<String>,
    pub theme_background: Option<String>,
    pub theme_selection_background: Option<String>,
    pub theme_subtext: Option<String>,
    pub workflow_description: Option<String>,
    pub workflow_uid: Option<String>,
    pub workflow_keyword: Option<String>,
    pub debug: bool,
}

/// ConfigProvider provides a strategy pattern solution for providing
/// the critical Alfred configuration data to a workflow.
pub trait ConfigProvider {
    fn config(&self) -> Result<WorkflowConfig>;
}

/// AlfredEnvProvider reads workflow configuration values from environment
/// variables set by the Alfred process.
///
/// This provider should be used for production code paths. It returns an
/// Err if any of the following required environment variables are not set:
///
/// alfred_workflow_cache
/// alfred_workflow_data
///
pub struct AlfredEnvProvider;

impl ConfigProvider for AlfredEnvProvider {
    fn config(&self) -> Result<WorkflowConfig> {
        let debug = env::var(VAR_DEBUG).unwrap_or_default();
        let debug = debug == "1" || debug.to_lowercase() == "true";

        let config = WorkflowConfig {
            // Required configuration values. Return Err if missing
            workflow_bundleid: env::var(VAR_WORKFLOW_BUNDLEID)?,
            workflow_cache: env::var(VAR_WORKFLOW_CACHE)?.into(),
            workflow_data: env::var(VAR_WORKFLOW_DATA)?.into(),
            version: env::var(VAR_VERSION)?,
            version_build: env::var(VAR_VERSION_BUILD)?,
            workflow_name: env::var(VAR_WORKFLOW_NAME)?,
            workflow_version: env::var(VAR_WORKFLOW_VERSION)?,

            // Optional configuration values. Set to blank defaults if not provided
            preferences: env::var(VAR_PREFERENCES).ok(),
            preferences_localhash: env::var(VAR_PREFERENCES_LOCALHASH).ok(),
            theme: env::var(VAR_THEME).ok(),
            theme_background: env::var(VAR_THEME_BACKGROUND).ok(),
            theme_selection_background: env::var(VAR_THEME_SELECTION_BACKGROUND).ok(),
            theme_subtext: env::var(VAR_THEME_SUBTEXT).ok(),
            workflow_description: env::var(VAR_WORKFLOW_DESCRIPTION).ok(),
            workflow_uid: env::var(VAR_WORKFLOW_UID).ok(),
            workflow_keyword: env::var(VAR_WORKFLOW_KEYWORD).ok(),
            debug,
        };
        Ok(config)
    }
}

/// TestingProvider implements a mocking strategy for ConfigProvider.
///
/// Given a PathBuf, it returns a WorkflowConfig that will operate
/// inside the provided directory. It will use workflow_data/ and
/// workflow_cache/ subdirectories within the provided directory.
/// All other required properties are set to hard-coded test values.
///
/// Typical usage is based around directories created by the tempfile crate
///
/// let dir = tempfile::tempdir().unwrap().into_path();
/// config::TestingProvider(dir)
///
pub struct TestingProvider(pub PathBuf);

impl ConfigProvider for TestingProvider {
    fn config(&self) -> Result<WorkflowConfig> {
        Ok(WorkflowConfig {
            preferences: Some("/Users/Crayons/Dropbox/Alfred/Alfred.alfredpreferences".to_string()),
            preferences_localhash: Some("adbd4f66bc3ae8493832af61a41ee609b20d8705".to_string()),
            theme: Some("alfred.theme.yosemite".to_string()),
            theme_background: Some("rgba(255,255,255,0.98)".to_string()),
            theme_selection_background: Some("rgba(255,255,255,0.98)".to_string()),
            theme_subtext: Some("3".to_string()),
            version: "5.0".to_string(),
            version_build: "2058".to_string(),
            workflow_bundleid: "com.alfredapp.googlesuggest".to_string(),
            workflow_cache: self.0.join("workflow_cache"),
            workflow_data: self.0.join("workflow_data"),
            workflow_name: "Test Workflow".to_string(),
            workflow_description: Some(
                "The description of the workflow we use for testing".to_string(),
            ),
            workflow_version: "1.7".to_string(),
            workflow_uid: Some("user.workflow.B0AC54EC-601C-479A-9428-01F9FD732959".to_string()),
            workflow_keyword: None,
            debug: true,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_alfred_env_provider_with_errors() {
        let provider = AlfredEnvProvider;
        let result = provider.config();
        assert!(result.is_err());
    }

    #[test]
    fn test_testing_provider() {
        let dir = tempfile::tempdir().unwrap().into_path();
        let provider = TestingProvider(dir);
        let config = provider.config().unwrap();
        assert_eq!(config.workflow_bundleid, "com.alfredapp.googlesuggest");
        assert_eq!(config.workflow_name, "Test Workflow");
        assert_eq!(config.version, "5.0");
        assert_eq!(config.version_build, "2058");
    }
}
