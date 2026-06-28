//! The invocation harness: build a `Simulator`, run workflows, get a `Screen`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::config::{
    read_plist_metadata, ConfigProvider, WorkflowConfig, DEFAULT_ALFRED_VERSION,
    DEFAULT_ALFRED_VERSION_BUILD,
};
use crate::error::WorkflowError;
use crate::workflow::{finalize_workflow, setup_workflow};
use crate::Runnable;

use super::action::ActionResult;
use super::graph::Severity;
use super::screen::{Screen, ScreenItem};
use super::{ObjectKind, WorkflowGraph};

/// A headless Alfred workflow simulator.
///
/// `Simulator` wraps a workflow directory (containing an `info.plist`) and provides
/// methods to invoke the workflow either in-process or via a subprocess, returning
/// a [`Screen`] with the rendered items.
///
/// # Defaults and Overrides
///
/// - **`workflow_dir`** — required; source of `info.plist` and the parsed graph.
/// - **`bundleid`** — defaults to the value from `info.plist`; overridable.
/// - **`cache_dir` / `data_dir`** — default to temp directories; overridable.
/// - **`binary`** — path to an already-built binary for subprocess invocation.
///
/// # Example
///
/// ```no_run
/// use alfrusco::simulator::Simulator;
///
/// let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
/// let screen = sim.invoke(&["fruits"]).unwrap();
/// assert_eq!(screen.items()[0].title(), "Apple");
/// ```
#[derive(Debug)]
pub struct Simulator {
    workflow_dir: PathBuf,
    graph: Arc<WorkflowGraph>,
    bundleid: String,
    workflow_name: String,
    cache_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    binary: Option<PathBuf>,
    source_uid: Option<String>,
}

impl Simulator {
    /// Creates a new simulator from a workflow directory containing `info.plist`.
    ///
    /// Parses the workflow's `info.plist` to extract the graph, bundle ID, and name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory does not contain `info.plist`
    /// - The plist cannot be parsed
    /// - The plist is missing a `bundleid`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use alfrusco::simulator::Simulator;
    ///
    /// let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    /// ```
    pub fn for_workflow_dir(dir: impl AsRef<Path>) -> Result<Self, SimulatorError> {
        let workflow_dir = dir.as_ref().to_path_buf();
        let plist_path = workflow_dir.join("info.plist");

        if !plist_path.is_file() {
            return Err(SimulatorError::NoPlist(workflow_dir));
        }

        let graph = WorkflowGraph::from_plist_file(&plist_path)
            .map_err(|e| SimulatorError::PlistParse(e.to_string()))?;

        let (bundleid, workflow_name) = read_plist_metadata(&plist_path)
            .ok_or_else(|| SimulatorError::PlistParse("missing bundleid".to_string()))?;

        Ok(Self {
            workflow_dir,
            graph: Arc::new(graph),
            bundleid,
            workflow_name,
            cache_dir: None,
            data_dir: None,
            binary: None,
            source_uid: None,
        })
    }

    /// Overrides the cache directory (default: temp directory).
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(path.into());
        self
    }

    /// Overrides the data directory (default: temp directory).
    pub fn data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Sets the path to the pre-built binary for subprocess invocation.
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = Some(path.into());
        self
    }

    /// Sets the source Script Filter UID for action routing.
    ///
    /// When a [`Screen`] is produced by this simulator, the source UID determines
    /// which graph node to walk from when resolving actions. If not set, defaults
    /// to the first keyword-bearing Script Filter in the graph.
    pub fn source_filter(mut self, uid: impl Into<String>) -> Self {
        self.source_uid = Some(uid.into());
        self
    }

    /// Returns a reference to the parsed workflow graph.
    pub fn graph(&self) -> &WorkflowGraph {
        &self.graph
    }

    /// Returns the workflow directory path.
    pub fn workflow_dir(&self) -> &Path {
        &self.workflow_dir
    }

    /// Returns the workflow's bundle ID.
    pub fn bundleid(&self) -> &str {
        &self.bundleid
    }

    /// Runs a [`Runnable`] workflow in-process and returns the rendered [`Screen`].
    ///
    /// This is the preferred mode for a workflow's own integration tests — no
    /// compiled binary or deployment is required.
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow fails to produce valid JSON output.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use alfrusco::simulator::Simulator;
    /// use alfrusco::{Runnable, Workflow, Item};
    ///
    /// struct MyWorkflow;
    /// impl Runnable for MyWorkflow {
    ///     type Error = alfrusco::Error;
    ///     fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
    ///         wf.append_item(Item::new("Hello"));
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
    /// let screen = sim.run_in_process(MyWorkflow).unwrap();
    /// assert_eq!(screen.items()[0].title(), "Hello");
    /// ```
    pub fn run_in_process<R: Runnable>(&self, runnable: R) -> Result<Screen, SimulatorError> {
        let provider = self.build_config_provider()?;
        let mut buffer = Vec::new();
        execute_to_buffer(&provider, runnable, &mut buffer);
        let json =
            String::from_utf8(buffer).map_err(|e| SimulatorError::OutputParse(e.to_string()))?;
        let screen =
            Screen::from_json(&json).map_err(|e| SimulatorError::OutputParse(e.to_string()))?;
        Ok(self.attach_context(screen))
    }

    /// Invokes the workflow binary as a subprocess with the given arguments
    /// and returns the rendered [`Screen`].
    ///
    /// Requires a binary path set via [`Simulator::binary`].
    ///
    /// Alfred environment variables are injected into the subprocess so the binary
    /// can resolve its configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No binary path has been set via [`Simulator::binary`]
    /// - The binary fails to execute
    /// - The output is not valid Script Filter JSON
    ///
    /// # Example
    ///
    /// ```no_run
    /// use alfrusco::simulator::Simulator;
    ///
    /// let sim = Simulator::for_workflow_dir("examples/menu_workflow")
    ///     .unwrap()
    ///     .binary("target/debug/examples/menu");
    /// let screen = sim.invoke(&["fruits"]).unwrap();
    /// assert_eq!(screen.items()[0].title(), "Apple");
    /// ```
    pub fn invoke(&self, args: &[&str]) -> Result<Screen, SimulatorError> {
        let binary = self.binary.clone().ok_or(SimulatorError::NoBinary)?;

        if !binary.is_file() {
            return Err(SimulatorError::BinaryNotFound(binary));
        }

        self.invoke_binary(&binary, args)
    }

    /// Invokes a specific Script Filter by its UID, using that filter's own
    /// `scriptfile` or `script` configuration from `info.plist`.
    ///
    /// This is the faithful invocation mode: it runs the exact command Alfred
    /// would run for this Script Filter, with the query passed as arguments.
    ///
    /// Resolution order:
    /// 1. If `scriptfile` is configured and exists at `<workflow_dir>/<scriptfile>`,
    ///    run it directly with the query as arguments.
    /// 2. If `script` is configured (inline shell command), execute it via `/bin/sh`
    ///    with `$1`/`{query}` substituted with the first query argument.
    /// 3. Fall back to the binary set via [`Simulator::binary`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The UID does not correspond to a Script Filter in the graph
    /// - No `scriptfile`, `script`, or fallback binary is available
    /// - The resolved binary does not exist
    /// - The binary/script fails to execute
    pub fn invoke_script_filter(
        &self,
        filter_uid: &str,
        args: &[&str],
    ) -> Result<Screen, SimulatorError> {
        let node = self.graph.objects().get(filter_uid).ok_or_else(|| {
            SimulatorError::OutputParse(format!("no object with UID '{filter_uid}'"))
        })?;

        // 1. Try scriptfile
        if let Some(script_file) = node.script_file() {
            let resolved = self.workflow_dir.join(script_file);
            if resolved.is_file() {
                let screen = self.invoke_binary(&resolved, args)?;
                return Ok(screen.with_context(Arc::clone(&self.graph), filter_uid.to_string()));
            }
        }

        // 2. Try inline script (execute via shell with $1 substitution)
        if let Some(inline_script) = node.script() {
            let screen = self.invoke_inline_script(inline_script, args)?;
            return Ok(screen.with_context(Arc::clone(&self.graph), filter_uid.to_string()));
        }

        // 3. Fall back to explicit binary
        if let Some(binary) = &self.binary {
            if !binary.is_file() {
                return Err(SimulatorError::BinaryNotFound(binary.clone()));
            }
            let screen = self.invoke_binary(binary, args)?;
            return Ok(screen.with_context(Arc::clone(&self.graph), filter_uid.to_string()));
        }

        Err(SimulatorError::NoBinary)
    }

    /// Performs a dynamic audit by invoking each keyword's Script Filter and
    /// checking that navigation items route correctly.
    ///
    /// A "navigation item" is one carrying a navigation signal:
    /// - Non-empty `variables` (typically used for drill-in state)
    /// - An `autocomplete` string (tab-completion loopback)
    ///
    /// For each such item, the audit verifies that its action route reaches
    /// another Script Filter (`DrilledIn` or `TypedAutocomplete`). If a nav item
    /// routes to `RanScript`, `OpenedUrl`, or `DeadEnd`, that's an error.
    ///
    /// Leaf items (no variables, no autocomplete) are NOT flagged even if they
    /// route to a Run Script or Open URL — that's expected behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if workflow invocation fails entirely. Individual
    /// misrouted items are reported as [`DynamicAuditDiagnostic`] entries.
    pub fn dynamic_audit(&self) -> Result<Vec<DynamicAuditDiagnostic>, SimulatorError> {
        let mut diagnostics = Vec::new();

        // Find all keyword-bearing Script Filters
        let filters: Vec<(&str, &str)> = self
            .graph
            .objects()
            .values()
            .filter(|n| n.kind == ObjectKind::ScriptFilter && n.keyword.is_some())
            .map(|n| (n.uid.as_str(), n.keyword.as_deref().unwrap_or("")))
            .collect();

        for (uid, keyword) in filters {
            // Invoke the Script Filter with no query (top-level)
            let screen = match self.invoke_script_filter(uid, &[]) {
                Ok(s) => s,
                Err(e) => {
                    diagnostics.push(DynamicAuditDiagnostic {
                        severity: Severity::Warning,
                        message: format!("Could not invoke Script Filter '{keyword}' ({uid}): {e}"),
                        keyword: keyword.to_string(),
                        item_title: None,
                    });
                    continue;
                }
            };

            // Check each item for navigation signals
            for (i, item) in screen.items().iter().enumerate() {
                if !Self::is_navigation_item(item) {
                    continue;
                }

                // This item has navigation signals — check its route
                if let Some(action) = screen.action(i) {
                    match &action {
                        ActionResult::DrilledIn { .. } | ActionResult::TypedAutocomplete { .. } => {
                            // Correct: nav item routes to another Script Filter
                        }
                        ActionResult::RanScript { target_uid } => {
                            diagnostics.push(DynamicAuditDiagnostic {
                                severity: Severity::Error,
                                message: format!(
                                    "Navigation item '{}' from Script Filter '{keyword}' ({uid}) \
                                     routes to Run Script ({target_uid}) instead of another Script Filter",
                                    item.title()
                                ),
                                keyword: keyword.to_string(),
                                item_title: Some(item.title().to_string()),
                            });
                        }
                        ActionResult::OpenedUrl { url_template } => {
                            diagnostics.push(DynamicAuditDiagnostic {
                                severity: Severity::Error,
                                message: format!(
                                    "Navigation item '{}' from Script Filter '{keyword}' ({uid}) \
                                     routes to Open URL ({url_template}) instead of another Script Filter",
                                    item.title()
                                ),
                                keyword: keyword.to_string(),
                                item_title: Some(item.title().to_string()),
                            });
                        }
                        ActionResult::DeadEnd => {
                            diagnostics.push(DynamicAuditDiagnostic {
                                severity: Severity::Error,
                                message: format!(
                                    "Navigation item '{}' from Script Filter '{keyword}' ({uid}) \
                                     has no route (dead-end)",
                                    item.title()
                                ),
                                keyword: keyword.to_string(),
                                item_title: Some(item.title().to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(diagnostics)
    }

    /// Returns `true` if a screen item carries navigation signals.
    ///
    /// Navigation signals indicate the item is meant to drill in to another
    /// Script Filter rather than being a leaf action:
    /// - Non-empty `variables` on the item (unless the arg is a URL — leaf actions
    ///   may carry variables for Alfred's use without being navigation items)
    /// - An `autocomplete` string set on the item
    fn is_navigation_item(item: &ScreenItem) -> bool {
        // Autocomplete is always a navigation signal
        if item.autocomplete().is_some() {
            return true;
        }
        // Variables are a navigation signal UNLESS the arg is a URL (leaf action)
        if let Some(vars) = item.raw().get("variables").and_then(|v| v.as_object()) {
            if !vars.is_empty() {
                // If the arg looks like a URL, this is a leaf action with metadata
                if let Some(arg) = item.arg() {
                    if arg.starts_with("http://") || arg.starts_with("https://") {
                        return false;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Internal: invoke a binary with args and Alfred env vars, return a Screen.
    fn invoke_binary(&self, binary: &Path, args: &[&str]) -> Result<Screen, SimulatorError> {
        let (cache_dir, data_dir) = self.resolve_dirs()?;

        let output = Command::new(binary)
            .args(args)
            .env("alfred_workflow_bundleid", &self.bundleid)
            .env("alfred_workflow_name", &self.workflow_name)
            .env(
                "alfred_workflow_cache",
                cache_dir.to_string_lossy().as_ref(),
            )
            .env("alfred_workflow_data", data_dir.to_string_lossy().as_ref())
            .env("alfred_version", DEFAULT_ALFRED_VERSION)
            .env("alfred_version_build", DEFAULT_ALFRED_VERSION_BUILD)
            .output()
            .map_err(|e| SimulatorError::BinaryExec(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SimulatorError::BinaryExec(format!(
                "exit code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr.trim()
            )));
        }

        let json = String::from_utf8(output.stdout)
            .map_err(|e| SimulatorError::OutputParse(e.to_string()))?;
        let screen =
            Screen::from_json(&json).map_err(|e| SimulatorError::OutputParse(e.to_string()))?;
        Ok(self.attach_context(screen))
    }

    /// Internal: invoke an inline shell script with `$1`/`{query}` substitution.
    ///
    /// Alfred expands `{query}` in inline scripts to the user's input and also
    /// passes it as `$1`. We emulate both: replace `{query}` in the script text
    /// and set `$1` via shell argument.
    fn invoke_inline_script(&self, script: &str, args: &[&str]) -> Result<Screen, SimulatorError> {
        let (cache_dir, data_dir) = self.resolve_dirs()?;
        let query = args.join(" ");

        // Substitute {query} in the script (Alfred's expansion)
        let expanded = script.replace("{query}", &query);

        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(&expanded)
            .arg("--") // separator
            .arg(&query) // becomes $1
            .current_dir(&self.workflow_dir)
            .env("alfred_workflow_bundleid", &self.bundleid)
            .env("alfred_workflow_name", &self.workflow_name)
            .env(
                "alfred_workflow_cache",
                cache_dir.to_string_lossy().as_ref(),
            )
            .env("alfred_workflow_data", data_dir.to_string_lossy().as_ref())
            .env("alfred_version", DEFAULT_ALFRED_VERSION)
            .env("alfred_version_build", DEFAULT_ALFRED_VERSION_BUILD)
            .output()
            .map_err(|e| SimulatorError::BinaryExec(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SimulatorError::BinaryExec(format!(
                "inline script exit code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr.trim()
            )));
        }

        let json = String::from_utf8(output.stdout)
            .map_err(|e| SimulatorError::OutputParse(e.to_string()))?;
        Screen::from_json(&json).map_err(|e| SimulatorError::OutputParse(e.to_string()))
    }

    /// Resolves cache and data directories (using overrides or temp defaults).
    fn resolve_dirs(&self) -> Result<(PathBuf, PathBuf), SimulatorError> {
        let cache_dir = self.cache_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir()
                .join("alfrusco-simulator")
                .join(&self.bundleid)
                .join("cache")
        });
        let data_dir = self.data_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir()
                .join("alfrusco-simulator")
                .join(&self.bundleid)
                .join("data")
        });
        std::fs::create_dir_all(&cache_dir).map_err(|e| SimulatorError::Io(e.to_string()))?;
        std::fs::create_dir_all(&data_dir).map_err(|e| SimulatorError::Io(e.to_string()))?;
        Ok((cache_dir, data_dir))
    }

    /// Builds a `ConfigProvider` from the simulator's settings.
    fn build_config_provider(&self) -> Result<SimulatorConfigProvider, SimulatorError> {
        let (cache_dir, data_dir) = self.resolve_dirs()?;
        Ok(SimulatorConfigProvider {
            bundleid: self.bundleid.clone(),
            workflow_name: self.workflow_name.clone(),
            cache_dir,
            data_dir,
        })
    }

    /// Attaches graph context to a screen for action routing.
    fn attach_context(&self, screen: Screen) -> Screen {
        let source_uid = self.resolve_source_uid();
        screen.with_context(Arc::clone(&self.graph), source_uid)
    }

    /// Resolves the source Script Filter UID (explicit override or first keyword-bearing filter).
    fn resolve_source_uid(&self) -> String {
        if let Some(uid) = &self.source_uid {
            return uid.clone();
        }
        // Default: first Script Filter with a keyword in the graph
        self.graph
            .objects()
            .values()
            .find(|n| n.kind == ObjectKind::ScriptFilter && n.keyword.is_some())
            .map(|n| n.uid.clone())
            .unwrap_or_default()
    }
}

/// A config provider that uses the simulator's resolved settings.
struct SimulatorConfigProvider {
    bundleid: String,
    workflow_name: String,
    cache_dir: PathBuf,
    data_dir: PathBuf,
}

impl ConfigProvider for SimulatorConfigProvider {
    fn config(&self) -> crate::Result<WorkflowConfig> {
        Ok(WorkflowConfig {
            workflow_bundleid: self.bundleid.clone(),
            workflow_cache: self.cache_dir.clone(),
            workflow_data: self.data_dir.clone(),
            version: DEFAULT_ALFRED_VERSION.to_string(),
            version_build: DEFAULT_ALFRED_VERSION_BUILD.to_string(),
            workflow_name: self.workflow_name.clone(),
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
            debug: false,
        })
    }
}

/// Runs a `Runnable` to a buffer (like `execute` but without `process::exit`
/// on internal handler dispatch).
fn execute_to_buffer<R: Runnable>(
    provider: &dyn ConfigProvider,
    runnable: R,
    buffer: &mut Vec<u8>,
) {
    let mut workflow = setup_workflow(provider);
    if let Err(e) = runnable.run(&mut workflow) {
        workflow.prepend_item(e.error_item());
    }
    finalize_workflow(workflow, buffer);
}

/// Errors from the simulator.
#[derive(Debug, thiserror::Error)]
pub enum SimulatorError {
    /// The workflow directory does not contain `info.plist`.
    #[error("no info.plist found in workflow directory: {0}")]
    NoPlist(PathBuf),

    /// The `info.plist` could not be parsed.
    #[error("failed to parse info.plist: {0}")]
    PlistParse(String),

    /// No binary path was set for subprocess invocation.
    #[error("no binary path set (use .binary() to specify one)")]
    NoBinary,

    /// The specified binary does not exist.
    #[error("binary not found: {0}")]
    BinaryNotFound(PathBuf),

    /// The binary failed to execute.
    #[error("binary execution failed: {0}")]
    BinaryExec(String),

    /// The workflow output could not be parsed as Script Filter JSON.
    #[error("failed to parse workflow output: {0}")]
    OutputParse(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
}

/// A diagnostic from dynamic audit (invoking the workflow and inspecting item routing).
#[derive(Debug, Clone)]
pub struct DynamicAuditDiagnostic {
    /// Severity of this finding.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub message: String,
    /// The keyword of the Script Filter that produced the item.
    pub keyword: String,
    /// The title of the misrouted item, if applicable.
    pub item_title: Option<String>,
}
