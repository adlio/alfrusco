pub use cache::cache_dir;
pub use clipboard::copy_markdown_link_to_clipboard;
pub use clipboard::copy_rich_text_link_to_clipboard;
pub use data::data_dir;
pub use error::{Error, Result, WorkflowError};
pub use icon::{Icon, *};
pub use item::{filter_and_sort_items, Item, Key, Modifier};
pub use response::Response;
pub use url_item::URLItem;
pub use workflow::Workflow;

mod background;
mod background_job;
mod cache;
mod clipboard;
pub mod config;
mod data;
mod error;
mod icon;
mod item;
mod response;
mod url_item;
mod workflow;

pub fn handle() {
    clipboard::handle_clipboard()
}
