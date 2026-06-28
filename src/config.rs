use std::env;
use std::path::{Path, PathBuf};

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

pub(crate) const DEFAULT_ALFRED_VERSION: &str = "5.5";
pub(crate) const DEFAULT_ALFRED_VERSION_BUILD: &str = "2300";

/// `WorkflowConfig` holds the configuration values for the current workflow.
///
/// In a real-world scenario, these values are read from environment variables.
/// The `from_env()` constructor is the primary way to create a `WorkflowConfig`.
///
/// See <https://www.alfredapp.com/help/workflows/script-environment-variables/>
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WorkflowConfig {
    pub workflow_bundleid: String,
    pub workflow_cache: PathBuf,
    pub workflow_data: PathBuf,
    pub version: String,
    pub version_build: String,
    pub workflow_name: String,

    pub workflow_version: Option<String>,
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

/// `ConfigProvider` provides a strategy pattern solution for providing
/// the critical Alfred configuration data to a workflow.
pub trait ConfigProvider {
    fn config(&self) -> Result<WorkflowConfig>;
}

/// `AlfredEnvProvider` resolves workflow configuration using a three-tier fallback:
///
/// 1. **Environment variables** — if Alfred's env vars are present, use them (production path).
/// 2. **Infer from binary location** — locate an `info.plist` near the running executable,
///    read the `bundleid` and `name`, and derive standard Alfred directories from them.
/// 3. **Temp directory fallback** — if no `info.plist` is found, use temporary directories
///    for cache and data. An ERROR-level log is emitted to STDERR naming the exact paths.
///
/// This ensures an alfrusco workflow binary never panics on startup due to missing
/// environment variables, making it usable from the terminal, cron, CI, or an AI agent.
pub struct AlfredEnvProvider;

impl ConfigProvider for AlfredEnvProvider {
    fn config(&self) -> Result<WorkflowConfig> {
        let debug = env::var(VAR_DEBUG).unwrap_or_default();
        let debug = debug == "1" || debug.to_lowercase() == "true";

        // Tier 1: Try environment variables (full Alfred runtime)
        if let Some(config) = try_env_config(debug) {
            return Ok(config);
        }

        // Tier 2: Try inferring from binary location (deployed workflow directory)
        if let Some(config) = try_infer_from_binary(debug) {
            return Ok(config);
        }

        // Tier 3: Temp directory fallback
        Ok(temp_fallback_config(debug))
    }
}

/// Attempts to build config entirely from environment variables.
/// Returns `None` if any required variable is missing.
fn try_env_config(debug: bool) -> Option<WorkflowConfig> {
    Some(WorkflowConfig {
        workflow_bundleid: env::var(VAR_WORKFLOW_BUNDLEID).ok()?,
        workflow_cache: env::var(VAR_WORKFLOW_CACHE).ok()?.into(),
        workflow_data: env::var(VAR_WORKFLOW_DATA).ok()?.into(),
        version: env::var(VAR_VERSION).ok()?,
        version_build: env::var(VAR_VERSION_BUILD).ok()?,
        workflow_name: env::var(VAR_WORKFLOW_NAME).ok()?,
        workflow_version: env::var(VAR_WORKFLOW_VERSION).ok(),
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
    })
}

/// Attempts to infer configuration from the binary's location by finding an
/// adjacent `info.plist` file. Walks up to 3 parent levels from the executable.
fn try_infer_from_binary(debug: bool) -> Option<WorkflowConfig> {
    let exe_path = env::current_exe().ok()?;
    let plist_path = find_info_plist(&exe_path)?;
    let (bundleid, name) = read_plist_metadata(&plist_path)?;

    let home = dirs_for_bundleid(&bundleid);

    Some(WorkflowConfig {
        workflow_bundleid: bundleid,
        workflow_cache: home.cache,
        workflow_data: home.data,
        version: DEFAULT_ALFRED_VERSION.to_string(),
        version_build: DEFAULT_ALFRED_VERSION_BUILD.to_string(),
        workflow_name: name,
        workflow_version: None,
        preferences: None,
        preferences_localhash: None,
        theme: None,
        theme_background: None,
        theme_selection_background: None,
        theme_subtext: None,
        workflow_description: None,
        workflow_uid: None,
        workflow_keyword: None,
        debug,
    })
}

/// Produces a config using temp directories. Logs an ERROR to STDERR.
fn temp_fallback_config(debug: bool) -> WorkflowConfig {
    let binary_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "alfrusco-workflow".to_string());

    let base = env::temp_dir().join(&binary_name);
    let cache = base.join("cache");
    let data = base.join("data");

    eprintln!(
        "ERROR: alfrusco: No Alfred environment variables or info.plist found. \
         Using ephemeral temp directories (data will not persist):\n  \
         cache: {}\n  data: {}",
        cache.display(),
        data.display()
    );

    WorkflowConfig {
        workflow_bundleid: format!("com.alfrusco.{binary_name}"),
        workflow_cache: cache,
        workflow_data: data,
        version: DEFAULT_ALFRED_VERSION.to_string(),
        version_build: DEFAULT_ALFRED_VERSION_BUILD.to_string(),
        workflow_name: binary_name,
        workflow_version: None,
        preferences: None,
        preferences_localhash: None,
        theme: None,
        theme_background: None,
        theme_selection_background: None,
        theme_subtext: None,
        workflow_description: None,
        workflow_uid: None,
        workflow_keyword: None,
        debug,
    }
}

/// Searches for `info.plist` near the executable path.
///
/// Checks: the exe's directory, then up to 3 parent levels. At each level,
/// checks both `<dir>/info.plist` and `<dir>/workflow/info.plist`.
pub fn find_info_plist(exe_path: &Path) -> Option<PathBuf> {
    let exe_dir = exe_path.parent()?;

    // Check the exe's own directory and up to 3 levels above
    let mut dir = exe_dir.to_path_buf();
    for _ in 0..4 {
        let candidate = dir.join("info.plist");
        if candidate.is_file() {
            return Some(candidate);
        }
        let workflow_candidate = dir.join("workflow").join("info.plist");
        if workflow_candidate.is_file() {
            return Some(workflow_candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Reads `bundleid` and `name` from an Alfred `info.plist` file.
///
/// Returns `None` if the file cannot be parsed or lacks a `bundleid`.
pub fn read_plist_metadata(path: &Path) -> Option<(String, String)> {
    let value = plist::Value::from_file(path).ok()?;
    let dict = value.as_dictionary()?;
    let bundleid = dict.get("bundleid")?.as_string()?.to_string();
    let name = dict
        .get("name")
        .and_then(|v| v.as_string())
        .unwrap_or("Unnamed Workflow")
        .to_string();
    Some((bundleid, name))
}

/// Standard Alfred 5 directory paths for a given bundle ID.
struct WorkflowDirs {
    cache: PathBuf,
    data: PathBuf,
}

/// Derives Alfred's standard per-workflow directories from a bundle ID.
///
/// Alfred 5 uses:
/// - Cache: `~/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/<bundleid>`
/// - Data: `~/Library/Application Support/Alfred/Workflow Data/<bundleid>`
fn dirs_for_bundleid(bundleid: &str) -> WorkflowDirs {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let home = PathBuf::from(home);

    WorkflowDirs {
        cache: home
            .join("Library/Caches/com.runningwithcrayons.Alfred/Workflow Data")
            .join(bundleid),
        data: home
            .join("Library/Application Support/Alfred/Workflow Data")
            .join(bundleid),
    }
}

/// `TestingProvider` implements a mocking strategy for `ConfigProvider`.
///
/// Given a `PathBuf`, it returns a `WorkflowConfig` that will operate
/// inside the provided directory. It will use `workflow_data/` and
/// `workflow_cache/` subdirectories within the provided directory.
/// All other required properties are set to hard-coded test values.
///
/// Typical usage is based around directories created by the `tempfile` crate:
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use alfrusco::config;
/// let dir = tempfile::tempdir().unwrap().into_path();
/// let provider = config::TestingProvider(dir);
/// ```
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
            workflow_version: Some("1.7".to_string()),
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
    fn test_alfred_env_provider_with_all_env_vars() {
        temp_env::with_vars(
            [
                (VAR_WORKFLOW_CACHE, Some("/made/up/cache_dir")),
                (VAR_WORKFLOW_DATA, Some("/made/up/data_dir")),
                (VAR_WORKFLOW_BUNDLEID, Some("com.alfredapp.googlesuggest")),
                (VAR_VERSION, Some("5.0")),
                (VAR_VERSION_BUILD, Some("2058")),
                (VAR_WORKFLOW_NAME, Some("Test Workflow")),
                (VAR_WORKFLOW_VERSION, Some("1.7")),
                (VAR_DEBUG, Some("true")),
            ],
            || {
                let provider = AlfredEnvProvider;
                let result = provider.config();
                assert!(result.is_ok(), "{result:?}");
                let config = result.unwrap();
                assert_eq!(config.workflow_bundleid, "com.alfredapp.googlesuggest");
                assert_eq!(config.workflow_cache, PathBuf::from("/made/up/cache_dir"));
            },
        );
    }

    #[test]
    fn test_alfred_env_provider_falls_back_when_env_missing() {
        // With no Alfred env vars, the provider should NOT error —
        // it should produce a fallback config (tier 2 or 3).
        temp_env::with_vars(
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
                assert!(
                    result.is_ok(),
                    "Should not error with missing env: {result:?}"
                );
                let config = result.unwrap();
                // Should have gotten some fallback config
                assert!(!config.workflow_bundleid.is_empty());
                assert!(!config.workflow_name.is_empty());
            },
        );
    }

    #[test]
    fn test_find_info_plist_in_same_directory() {
        let dir = tempfile::tempdir().unwrap();
        let plist_path = dir.path().join("info.plist");
        std::fs::write(&plist_path, "").unwrap();

        let exe_path = dir.path().join("mybinary");
        let found = find_info_plist(&exe_path);
        assert_eq!(found, Some(plist_path));
    }

    #[test]
    fn test_find_info_plist_in_workflow_subdirectory() {
        let dir = tempfile::tempdir().unwrap();
        let workflow_dir = dir.path().join("workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let plist_path = workflow_dir.join("info.plist");
        std::fs::write(&plist_path, "").unwrap();

        let exe_path = dir.path().join("mybinary");
        let found = find_info_plist(&exe_path);
        assert_eq!(found, Some(plist_path));
    }

    #[test]
    fn test_find_info_plist_in_parent() {
        let dir = tempfile::tempdir().unwrap();
        let plist_path = dir.path().join("info.plist");
        std::fs::write(&plist_path, "").unwrap();

        let subdir = dir.path().join("bin");
        std::fs::create_dir_all(&subdir).unwrap();
        let exe_path = subdir.join("mybinary");
        let found = find_info_plist(&exe_path);
        assert_eq!(found, Some(plist_path));
    }

    #[test]
    fn test_find_info_plist_returns_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let exe_path = dir.path().join("mybinary");
        let found = find_info_plist(&exe_path);
        assert_eq!(found, None);
    }

    #[test]
    fn test_read_plist_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let plist_path = dir.path().join("info.plist");
        std::fs::write(
            &plist_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>bundleid</key>
    <string>com.example.test</string>
    <key>name</key>
    <string>My Test Workflow</string>
</dict>
</plist>"#,
        )
        .unwrap();

        let result = read_plist_metadata(&plist_path);
        assert_eq!(
            result,
            Some((
                "com.example.test".to_string(),
                "My Test Workflow".to_string()
            ))
        );
    }

    #[test]
    fn test_read_plist_metadata_missing_bundleid() {
        let dir = tempfile::tempdir().unwrap();
        let plist_path = dir.path().join("info.plist");
        std::fs::write(
            &plist_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>name</key>
    <string>No Bundle ID</string>
</dict>
</plist>"#,
        )
        .unwrap();

        let result = read_plist_metadata(&plist_path);
        assert_eq!(result, None);
    }

    #[test]
    fn test_testing_provider() {
        let dir = tempfile::tempdir().unwrap().keep();
        let provider = TestingProvider(dir);
        let config = provider.config().unwrap();
        assert_eq!(config.workflow_bundleid, "com.alfredapp.googlesuggest");
        assert_eq!(config.workflow_name, "Test Workflow");
        assert_eq!(config.version, "5.0");
        assert_eq!(config.version_build, "2058");
    }
}
