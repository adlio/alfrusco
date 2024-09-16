mod background;
mod background_job;
mod clipboard;
mod config;
mod error;
mod icon;
mod item;
mod response;
mod url_item;
mod workflow;

pub use clipboard::copy_markdown_link_to_clipboard;
pub use clipboard::copy_rich_text_link_to_clipboard;
pub use error::{Error, Result, WorkflowError, WorkflowResult};
pub use icon::{Icon, *};
pub use item::{filter_and_sort_items, Item, Key, Modifier};
pub use response::Response;
pub use url_item::URLItem;
pub use workflow::{AsyncRunnable, Runnable, Workflow};
pub use workflow_config::WorkflowConfig;

pub fn handle() {
    clipboard::handle_clipboard()
}
