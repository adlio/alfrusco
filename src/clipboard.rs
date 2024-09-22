use log::{debug, info};
use std::env::var;
use std::process::Command;

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use hex::encode;

use crate::Response;
use crate::Result;

pub fn handle_clipboard() {
    let cmd = var("ALFRUSCO_COMMAND").ok();
    let title = var("TITLE").ok();
    let url = var("URL").ok();
    if let Some(cmd) = cmd {
        debug!("ALFRUSCO_COMMAND provided. Alfrusco will handle this request");

        if cmd == "richtext" || cmd == "markdown" {
            if let (Some(title), Some(url)) = (title, url) {
                if cmd == "richtext" {
                    copy_rich_text_link_to_clipboard(title, url);
                } else if cmd == "markdown" {
                    copy_markdown_link_to_clipboard(title, url);
                }
                write_empty_items().unwrap();
                std::process::exit(0);
            }
        }
    }
}

pub fn write_empty_items() -> Result<()> {
    Response::new_with_items(vec![])
        .write(std::io::stdout())
        .unwrap();
    Ok(())
}

pub fn copy_markdown_link_to_clipboard(title: impl Into<String>, url: impl Into<String>) {
    let markdown = format!("[{}]({})", title.into(), url.into());
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(markdown.clone()).unwrap();
    info!("wrote Markdown: {} to the clipboard", markdown);
}

pub fn copy_rich_text_link_to_clipboard(title: impl Into<String>, url: impl Into<String>) {
    let html = format!("<a href=\"{}\">{}</a>", url.into(), title.into());

    let apple_script = format!(
        "set the clipboard to {{text:\" \", «class HTML»:«data HTML{}»}}",
        encode(html.as_bytes()),
    );

    // Prepare and execute the osascript command
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&apple_script)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("osascript command failed: {}", stderr);
    }

    info!("wrote HTML to the clipboard as rich text: {}", html);
}
