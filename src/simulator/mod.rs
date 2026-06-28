//! Headless Alfred workflow simulator.
//!
//! This module provides tools for parsing, navigating, and testing Alfred workflows
//! without the Alfred UI. It includes:
//!
//! - [`WorkflowGraph`] — a parsed representation of an Alfred `info.plist`, modeling
//!   objects (Script Filters, actions, utilities) and their connections.
//! - Graph queries: reachability, keyword lookup, and navigation auditing.
//!
//! # Example
//!
//! ```no_run
//! use alfrusco::simulator::WorkflowGraph;
//!
//! let graph = WorkflowGraph::from_plist_file("workflow/info.plist").unwrap();
//! let uid = graph.script_filter_uid("menu").unwrap();
//! assert!(graph.reaches_script_filter(&uid));
//! ```

mod graph;

pub use graph::{AuditDiagnostic, Connection, ObjectKind, ObjectNode, Severity, WorkflowGraph};
