use anyhow::Result;
use clap::Parser;

use log::info;
use std::process::Command;

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use hex::encode;

use crate::Response;

#[derive(Parser, Debug)]
struct ClipboardCli {
    #[arg(short, long, env)]
    title: Option<String>,

    #[arg(short, long, env)]
    url: Option<String>,

    #[arg(short, long, env)]
    alfrusco_command: Option<String>,
}

pub fn handle_clipboard() {
    info!("handle_clipboard checking...");
    let args = ClipboardCli::parse();
    if let Some(command) = args.alfrusco_command {
        if let Some(title) = args.title {
            if let Some(url) = args.url {
                info!("alfrusco handling clipboard command: {}", command);
                if command == "richtext" {
                    copy_rich_text_link_to_clipboard(title, url);
                    write_empty_items().unwrap();
                    std::process::exit(0);
                } else if command == "markdown" {
                    copy_markdown_link_to_clipboard(title, url);
                    write_empty_items().unwrap();
                    std::process::exit(0);
                }
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
