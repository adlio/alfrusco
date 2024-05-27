mod cache;
mod clipboard;
pub mod config;
mod data;
mod item;
mod response;

pub use item::{filter_and_sort_items, Item, Key, Modifier, URLItem};
pub use response::Response;

pub use cache::cache_dir;
pub use data::data_dir;

pub use clipboard::copy_markdown_link_to_clipboard;
pub use clipboard::copy_rich_text_link_to_clipboard;

pub fn handle() {
    clipboard::handle_clipboard()
}
