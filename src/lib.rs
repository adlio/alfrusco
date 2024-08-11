pub use cache::cache_dir;
pub use clipboard::copy_markdown_link_to_clipboard;
pub use clipboard::copy_rich_text_link_to_clipboard;
pub use data::data_dir;
pub use error::{Error, Result};
pub use item::{filter_and_sort_items, Icon, Item, Key, Modifier};
pub use response::Response;
pub use url_item::URLItem;
pub use workflow::Workflow;
pub use workflow_config::WorkflowConfig;

mod background;
mod background_job;
mod cache;
mod clipboard;
pub mod config;
mod data;
mod error;
mod item;
mod response;
mod url_item;
mod workflow;
mod workflow_config;

pub fn handle() {
    clipboard::handle_clipboard()
}
