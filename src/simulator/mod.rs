//! Headless Alfred workflow simulator.
//!
//! This module provides tools for parsing, navigating, and testing Alfred workflows
//! without the Alfred UI. It includes:
//!
//! - [`WorkflowGraph`] — a parsed representation of an Alfred `info.plist`, modeling
//!   objects (Script Filters, actions, utilities) and their connections.
//! - [`Simulator`] — an invocation harness that runs workflows in-process or via
//!   subprocess and returns a [`Screen`].
//! - [`Screen`] / [`ScreenItem`] — parsed Script Filter output for inspection.
//! - [`ActionResult`] — the outcome of actioning an item (drill-in, open URL, etc.).
//! - Graph queries: reachability, keyword lookup, and navigation auditing.
//!
//! # Writing behavioural tests
//!
//! The primary use case is testing workflow navigation headlessly. Workflow authors
//! create a [`Simulator`] pointed at their workflow directory (which contains the
//! real `info.plist`), then call [`Simulator::run_in_process`] with their `Runnable`
//! to get a [`Screen`]. Each screen supports action routing through the graph:
//!
//! ```no_run
//! use alfrusco::simulator::{ActionResult, Simulator};
//! use alfrusco::{Item, Runnable, Workflow};
//!
//! struct MyWorkflow { query: String }
//! impl Runnable for MyWorkflow {
//!     type Error = alfrusco::Error;
//!     fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
//!         wf.append_item(Item::new("Fruits").arg("fruits").valid(true));
//!         Ok(())
//!     }
//! }
//!
//! // Point at your workflow directory (containing info.plist)
//! let sim = Simulator::for_workflow_dir("examples/menu_workflow").unwrap();
//!
//! // Run in-process — no compiled binary or deployment needed
//! let screen = sim.run_in_process(MyWorkflow { query: "".into() }).unwrap();
//! screen.assert_renders();
//!
//! // Verify action routing through the plist graph
//! let result = screen.action_first().unwrap();
//! result.assert_drills_in();
//! ```
//!
//! # Parsing and inspecting screens directly
//!
//! For unit-test assertions that don't need graph routing, parse JSON directly:
//!
//! ```
//! use alfrusco::simulator::Screen;
//!
//! let json = r#"{"items":[
//!     {"title":"Apple","subtitle":"A fruit","arg":"apple","valid":true},
//!     {"title":"Back","valid":false,"autocomplete":""}
//! ]}"#;
//!
//! let screen = Screen::from_json(json).unwrap();
//! assert_eq!(screen.len(), 2);
//! assert_eq!(screen.items()[0].title(), "Apple");
//! assert_eq!(screen.items()[0].arg(), Some("apple"));
//! assert!(screen.items()[0].is_valid());
//! assert!(!screen.items()[1].is_valid());
//! ```
//!
//! # Graph auditing
//!
//! Use [`WorkflowGraph`] to statically audit your workflow's navigation:
//!
//! ```no_run
//! use alfrusco::simulator::WorkflowGraph;
//!
//! let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
//!
//! // Check that a keyword reaches its action chain
//! assert!(graph.reaches_script_filter("SF-MAIN-001"));
//!
//! // Audit reports navigation defects (dead-ends, dangling connections)
//! let diagnostics = graph.audit_navigation(&["menu"]);
//! assert!(diagnostics.is_empty(), "unexpected defects: {diagnostics:?}");
//! ```

mod action;
mod graph;
mod harness;
mod screen;

pub use action::ActionResult;
pub use graph::{AuditDiagnostic, Connection, ObjectKind, ObjectNode, Severity, WorkflowGraph};
pub use harness::{DynamicAuditDiagnostic, Simulator, SimulatorError};
pub use screen::{Screen, ScreenError, ScreenItem};
