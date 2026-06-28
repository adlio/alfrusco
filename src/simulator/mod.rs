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
//! # Example
//!
//! ```no_run
//! use alfrusco::simulator::{Simulator, WorkflowGraph};
//!
//! // Parse and audit the workflow graph
//! let graph = WorkflowGraph::from_plist_file("examples/menu_workflow/info.plist").unwrap();
//! assert!(graph.reaches_script_filter("SF-MAIN-001"));
//!
//! // Run a binary and inspect results
//! let sim = Simulator::for_workflow_dir("examples/menu_workflow")
//!     .unwrap()
//!     .binary("target/debug/examples/menu");
//! let screen = sim.invoke(&["fruits"]).unwrap();
//! assert_eq!(screen.items()[0].title(), "Apple");
//! ```

mod action;
mod graph;
mod harness;
mod screen;

pub use action::ActionResult;
pub use graph::{AuditDiagnostic, Connection, ObjectKind, ObjectNode, Severity, WorkflowGraph};
pub use harness::{Simulator, SimulatorError};
pub use screen::{Screen, ScreenError, ScreenItem};
