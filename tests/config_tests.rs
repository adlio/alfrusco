use std::path::PathBuf;

use alfrusco::config::{AlfredEnvProvider, ConfigProvider, TestingProvider};
use temp_env::with_vars;
use tempfile::TempDir;

// Constants for environment variables
const VAR_WORKFLOW_BUNDLEID: &str = "alfred_workflow_bundleid";
const VAR_WORKFLOW_CACHE: &str = "alfred_workflow_cache";
const VAR_WORKFLOW_DATA: &str = "alfred_workflow_data";
const VAR_VERSION: &str = "alfred_version";
const VAR_VERSION_BUILD: &str = "alfred_version_build";
const VAR_WORKFLOW_NAME: &str = "alfred_workflow_name";
const VAR_DEBUG: &str = "alfred_debug";
const VAR_WORKFLOW_VERSION: &str = "alfred_workflow_version";
const VAR_PREFERENCES: &str = "alfred_preferences";
const VAR_PREFERENCES_LOCALHASH: &str = "alfred_preferences_localhash";
const VAR_THEME: &str = "alfred_theme";
const VAR_THEME_BACKGROUND: &str = "alfred_theme_background";
const VAR_THEME_SELECTION_BACKGROUND: &str = "alfred_theme_selection_background";
const VAR_THEME_SUBTEXT: &str = "alfred_theme_subtext";
const VAR_WORKFLOW_DESCRIPTION: &str = "alfred_workflow_description";
const VAR_WORKFLOW_UID: &str = "alfred_workflow_uid";
const VAR_WORKFLOW_KEYWORD: &str = "alfred_workflow_keyword";

#[test]
fn test_alfred_env_provider_missing_vars_falls_back() {
    // With env vars partially missing, AlfredEnvProvider should NOT error —
    // it falls back to tier 2 (info.plist) or tier 3 (temp dirs).
    with_vars(
        [
            (VAR_WORKFLOW_CACHE, Some("/made/up/cache_dir")),
            (VAR_WORKFLOW_DATA, Some("/made/up/data_dir")),
            (VAR_WORKFLOW_BUNDLEID, None::<&str>), // Missing bundleid
            (VAR_VERSION, Some("5.0")),
            (VAR_VERSION_BUILD, Some("2058")),
            (VAR_WORKFLOW_NAME, Some("Test Workflow")),
        ],
        || {
            let provider = AlfredEnvProvider;
            let result = provider.config();
            // Should succeed via fallback, not error
            assert!(result.is_ok(), "Expected fallback, got error: {result:?}");
        },
    );
}

#[test]
fn test_alfred_env_provider_no_env_vars_at_all() {
    // With NO Alfred env vars, the provider should produce a usable fallback config.
    with_vars(
        [
            (VAR_WORKFLOW_CACHE, None::<&str>),
            (VAR_WORKFLOW_DATA, None::<&str>),
            (VAR_WORKFLOW_BUNDLEID, None::<&str>),
            (VAR_VERSION, None::<&str>),
            (VAR_VERSION_BUILD, None::<&str>),
            (VAR_WORKFLOW_NAME, None::<&str>),
            (VAR_DEBUG, None::<&str>),
        ],
        || {
            let provider = AlfredEnvProvider;
            let result = provider.config();
            assert!(result.is_ok(), "Expected fallback, got error: {result:?}");
            let config = result.unwrap();
            assert!(!config.workflow_bundleid.is_empty());
            assert!(!config.workflow_name.is_empty());
        },
    );
}

#[test]
fn test_alfred_env_provider_with_all_optional_vars() {
    with_vars(
        [
            // Required vars
            (VAR_WORKFLOW_CACHE, Some("/made/up/cache_dir")),
            (VAR_WORKFLOW_DATA, Some("/made/up/data_dir")),
            (VAR_WORKFLOW_BUNDLEID, Some("com.test.workflow")),
            (VAR_VERSION, Some("5.0")),
            (VAR_VERSION_BUILD, Some("2058")),
            (VAR_WORKFLOW_NAME, Some("Test Workflow")),
            // Optional vars
            (VAR_DEBUG, Some("true")),
            (VAR_WORKFLOW_VERSION, Some("1.0")),
            (VAR_PREFERENCES, Some("/path/to/prefs")),
            (VAR_PREFERENCES_LOCALHASH, Some("hash123")),
            (VAR_THEME, Some("theme.name")),
            (VAR_THEME_BACKGROUND, Some("rgba(255,255,255,0.98)")),
            (
                VAR_THEME_SELECTION_BACKGROUND,
                Some("rgba(255,255,255,0.98)"),
            ),
            (VAR_THEME_SUBTEXT, Some("3")),
            (VAR_WORKFLOW_DESCRIPTION, Some("Test Description")),
            (VAR_WORKFLOW_UID, Some("test.uid")),
            (VAR_WORKFLOW_KEYWORD, Some("test")),
        ],
        || {
            let provider = AlfredEnvProvider;
            let result = provider.config();
            assert!(result.is_ok());

            let config = result.unwrap();
            assert_eq!(config.workflow_bundleid, "com.test.workflow");
            assert_eq!(config.workflow_cache, PathBuf::from("/made/up/cache_dir"));
            assert_eq!(config.workflow_data, PathBuf::from("/made/up/data_dir"));
            assert_eq!(config.version, "5.0");
            assert_eq!(config.version_build, "2058");
            assert_eq!(config.workflow_name, "Test Workflow");

            // Optional fields
            assert!(config.debug);
            assert_eq!(config.workflow_version, Some("1.0".to_string()));
            assert_eq!(config.preferences, Some("/path/to/prefs".to_string()));
            assert_eq!(config.preferences_localhash, Some("hash123".to_string()));
            assert_eq!(config.theme, Some("theme.name".to_string()));
            assert_eq!(
                config.theme_background,
                Some("rgba(255,255,255,0.98)".to_string())
            );
            assert_eq!(
                config.theme_selection_background,
                Some("rgba(255,255,255,0.98)".to_string())
            );
            assert_eq!(config.theme_subtext, Some("3".to_string()));
            assert_eq!(
                config.workflow_description,
                Some("Test Description".to_string())
            );
            assert_eq!(config.workflow_uid, Some("test.uid".to_string()));
            assert_eq!(config.workflow_keyword, Some("test".to_string()));
        },
    );
}

#[test]
fn test_alfred_env_provider_debug_false() {
    with_vars(
        [
            // Required vars
            (VAR_WORKFLOW_CACHE, Some("/made/up/cache_dir")),
            (VAR_WORKFLOW_DATA, Some("/made/up/data_dir")),
            (VAR_WORKFLOW_BUNDLEID, Some("com.test.workflow")),
            (VAR_VERSION, Some("5.0")),
            (VAR_VERSION_BUILD, Some("2058")),
            (VAR_WORKFLOW_NAME, Some("Test Workflow")),
            // Debug set to false
            (VAR_DEBUG, Some("false")),
        ],
        || {
            let provider = AlfredEnvProvider;
            let result = provider.config();
            assert!(result.is_ok());

            let config = result.unwrap();
            assert!(!config.debug);
        },
    );
}

#[test]
fn test_testing_provider_paths() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    let provider = TestingProvider(temp_path.clone());
    let config = provider.config().unwrap();

    // Check that the paths are correctly set
    assert_eq!(config.workflow_cache, temp_path.join("workflow_cache"));
    assert_eq!(config.workflow_data, temp_path.join("workflow_data"));
}

#[test]
fn test_workflow_config_clone() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    let provider = TestingProvider(temp_path);
    let config = provider.config().unwrap();

    // Test that we can clone the config
    let cloned_config = config.clone();

    // Verify the clone has the same values
    assert_eq!(config.workflow_bundleid, cloned_config.workflow_bundleid);
    assert_eq!(config.workflow_cache, cloned_config.workflow_cache);
    assert_eq!(config.workflow_data, cloned_config.workflow_data);
    assert_eq!(config.version, cloned_config.version);
    assert_eq!(config.version_build, cloned_config.version_build);
    assert_eq!(config.workflow_name, cloned_config.workflow_name);
    assert_eq!(config.debug, cloned_config.debug);
}
