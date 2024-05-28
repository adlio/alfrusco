mod cache;
mod clipboard;
pub mod config;
mod data;
mod error;
mod item;
mod response;
mod url_item;

pub use item::{filter_and_sort_items, Icon, Item, Key, Modifier};
pub use response::Response;
pub use url_item::URLItem;

pub use cache::cache_dir;
pub use data::data_dir;
pub use error::{Error, Result};

pub use clipboard::copy_markdown_link_to_clipboard;
pub use clipboard::copy_rich_text_link_to_clipboard;

pub fn handle() {
    clipboard::handle_clipboard()
}
