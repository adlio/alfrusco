//! `alfrusco-simulator` CLI — headless Alfred workflow auditing and navigation.
//!
//! Subcommands:
//! - `audit <dir>` — statically audit the workflow graph for navigation defects.
//! - `walk <dir> --binary <path> [query...]` — invoke the workflow and display results + routing.

use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use alfrusco::simulator::{ActionResult, Simulator, WorkflowGraph};

#[derive(Parser)]
#[command(
    name = "alfrusco-simulator",
    about = "Headless Alfred workflow simulator"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Audit a workflow's info.plist for navigation defects.
    Audit {
        /// Path to the workflow directory containing info.plist.
        dir: PathBuf,
    },
    /// Invoke a workflow and display rendered items with action routing.
    Walk {
        /// Path to the workflow directory containing info.plist.
        dir: PathBuf,

        /// Path to the pre-built workflow binary.
        #[arg(long)]
        binary: PathBuf,

        /// UID of the source Script Filter for action routing.
        /// Defaults to the first keyword-bearing Script Filter.
        #[arg(long)]
        source_filter: Option<String>,

        /// Query arguments passed to the workflow binary.
        query: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit { dir } => run_audit(&dir),
        Commands::Walk {
            dir,
            binary,
            source_filter,
            query,
        } => run_walk(&dir, &binary, source_filter.as_deref(), &query),
    }
}

fn run_audit(dir: &Path) {
    let plist_path = dir.join("info.plist");
    let graph = match WorkflowGraph::from_plist_file(&plist_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ERROR: failed to parse {}: {e}", plist_path.display());
            process::exit(1);
        }
    };

    // Collect all keywords from the graph
    let keywords: Vec<String> = graph
        .objects()
        .values()
        .filter_map(|n| n.keyword.clone())
        .collect();

    println!("Auditing: {}", plist_path.display());
    println!(
        "Objects: {}  Connections: {}  Keywords: {}",
        graph.objects().len(),
        graph.connections().len(),
        keywords.len()
    );
    println!();

    if keywords.is_empty() {
        println!("WARNING: no keywords found in workflow graph.");
        return;
    }

    let keyword_refs: Vec<&str> = keywords.iter().map(String::as_str).collect();
    let diagnostics = graph.audit_navigation(&keyword_refs);

    if diagnostics.is_empty() {
        println!("✓ No navigation defects found.");
    } else {
        println!("Found {} issue(s):", diagnostics.len());
        println!();
        for d in &diagnostics {
            println!("  [{:?}] {}", d.severity, d.message);
        }
        process::exit(1);
    }
}

fn run_walk(dir: &Path, binary: &Path, source_filter: Option<&str>, query: &[String]) {
    let sim = match Simulator::for_workflow_dir(dir) {
        Ok(s) => s.binary(binary),
        Err(e) => {
            eprintln!("ERROR: {e}");
            process::exit(1);
        }
    };

    let sim = if let Some(uid) = source_filter {
        sim.source_filter(uid)
    } else {
        sim
    };

    let args: Vec<&str> = query.iter().map(String::as_str).collect();
    let screen = match sim.invoke(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: workflow invocation failed: {e}");
            process::exit(1);
        }
    };

    println!(
        "Walk: {} --binary {} {}",
        dir.display(),
        binary.display(),
        query.join(" ")
    );
    println!("Items: {}", screen.len());
    println!();

    for (i, item) in screen.items().iter().enumerate() {
        let valid = if item.is_valid() { "✓" } else { "○" };
        println!("  [{i}] {valid} {}", item.title());
        if let Some(sub) = item.subtitle() {
            println!("      subtitle: {sub}");
        }
        if let Some(arg) = item.arg() {
            println!("      arg: {arg}");
        }
        if let Some(ac) = item.autocomplete() {
            println!("      autocomplete: {ac}");
        }

        // Show action routing
        if let Some(action) = screen.action(i) {
            let routing = match &action {
                ActionResult::DrilledIn { target_uid } => {
                    format!("→ DrilledIn({target_uid})")
                }
                ActionResult::OpenedUrl { url_template } => {
                    format!("→ OpenedUrl({url_template})")
                }
                ActionResult::RanScript { target_uid } => {
                    format!("→ RanScript({target_uid})")
                }
                ActionResult::TypedAutocomplete { text } => {
                    format!("→ TypedAutocomplete(\"{text}\")")
                }
                ActionResult::DeadEnd => "→ DeadEnd ⚠".to_string(),
            };
            println!("      route: {routing}");
        }
        println!();
    }
}
