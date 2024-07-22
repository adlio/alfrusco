use std::env;

use crate::Result;

const VarWorkflowName: &str = "alfred_workflow_name";
const VarWorkflowBundleID: &str = "alfred_workflow_bundleid";
const VarWorkflowVersion: &str = "alfred_workflow_version";
const VarWorkflowUID: &str = "alfred_workflow_uid";
const VarWorkflowCacheDir: &str = "alfred_workflow_cache";
const VarWorkflowDataDir: &str = "alfred_workflow_data";
const VarDebug: &str = "alfred_debug";
const VarTheme: &str = "alfred_theme";
const VarThemeBG: &str = "alfred_theme_background";
const VarThemeSelectionBG: &str = "alfred_theme_selection_background";
const VarVersion: &str = "alfred_version";
const VarVersionBuild: &str = "alfred_version_build";
const VarPreferences: &str = "alfred_preferences";
const VarPreferencesLocalHash: &str = "alfred_preferences_localhash";

/// Retrives a string-formatted value from the Workflow configuration
pub fn get_string(key: String) -> Result<String> {
    env::var(key).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_string() {
        env::set_var("TEST_KEY", "test_value");
        assert_eq!(get_string("TEST_KEY".to_string()).unwrap(), "test_value");
    }
}
