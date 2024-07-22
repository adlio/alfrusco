use alfrusco::{Response, URLItem};

pub fn main() {
    alfrusco::handle();

    let mut response = Response::new();
    response.skip_knowledge(true);
    response.items(vec![
        URLItem::new("DuckDuckGo", "https://www.duckduckgo.com").into(),
        URLItem::new("Google", "https://www.google.com").into(),
    ]);

    match response.write(std::io::stdout()) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error writing response: {}", e);
            std::process::exit(1);
        }
    }
}
