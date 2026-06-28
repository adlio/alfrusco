//! A hierarchical menu workflow demonstrating drill-in navigation.
//!
//! This example shows how to build a two-level menu in Alfred using the
//! variables/loopback pattern:
//!
//! 1. Top level: shows categories (items set a `category` variable and use autocomplete)
//! 2. Sub level: when a category is selected, shows items within that category
//!
//! The workflow's `info.plist` (in `examples/menu_workflow/`) wires a Conditional
//! object between the main Script Filter and a sub-Script Filter, routing based
//! on whether `{var:category}` is set.
//!
//! Run with:
//! ```sh
//! cargo run --example menu
//! cargo run --example menu -- fruits
//! ```

use alfrusco::config::AlfredEnvProvider;
use alfrusco::{execute, Item, Runnable, Workflow};
use clap::Parser;

/// Menu categories and their items.
const CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    (
        "fruits",
        &[
            ("Apple", "https://en.wikipedia.org/wiki/Apple"),
            ("Banana", "https://en.wikipedia.org/wiki/Banana"),
            ("Cherry", "https://en.wikipedia.org/wiki/Cherry"),
        ],
    ),
    (
        "colors",
        &[
            ("Red", "https://en.wikipedia.org/wiki/Red"),
            ("Green", "https://en.wikipedia.org/wiki/Green"),
            ("Blue", "https://en.wikipedia.org/wiki/Blue"),
        ],
    ),
    (
        "planets",
        &[
            ("Mars", "https://en.wikipedia.org/wiki/Mars"),
            ("Venus", "https://en.wikipedia.org/wiki/Venus"),
            ("Saturn", "https://en.wikipedia.org/wiki/Saturn"),
        ],
    ),
];

#[derive(Parser)]
#[command(name = "menu")]
struct MenuWorkflow {
    /// The query (optionally a category name to drill into).
    query: Vec<String>,
}

impl Runnable for MenuWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let query = self.query.join(" ");

        // Check if we're drilling into a category
        let category = std::env::var("category").ok().filter(|s| !s.is_empty());

        match category.as_deref().or_else(|| {
            // If the query exactly matches a category name, treat it as drill-in
            CATEGORIES
                .iter()
                .find(|(name, _)| *name == query)
                .map(|(name, _)| *name)
        }) {
            Some(cat) => {
                // Sub-level: show items in the selected category
                if let Some((_, items)) = CATEGORIES.iter().find(|(name, _)| *name == cat) {
                    for (title, url) in *items {
                        workflow.append_item(
                            Item::new(*title)
                                .subtitle(format!("Open {url}"))
                                .arg(*url)
                                .valid(true),
                        );
                    }
                } else {
                    workflow.append_item(
                        Item::new(format!("Unknown category: {cat}"))
                            .subtitle("No items found")
                            .valid(false),
                    );
                }
            }
            None => {
                // Top-level: show categories
                workflow.set_filter_keyword(query);
                for (name, items) in CATEGORIES {
                    workflow.append_item(
                        Item::new(capitalize(name))
                            .subtitle(format!("{} items", items.len()))
                            .arg(*name)
                            .var("category", *name)
                            .autocomplete(*name)
                            .valid(true),
                    );
                }
            }
        }

        Ok(())
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn main() {
    let _ = alfrusco::init_logging(&AlfredEnvProvider);
    let command = MenuWorkflow::parse();
    execute(&AlfredEnvProvider, command, &mut std::io::stdout());
}
