use alfrusco::{Item, Response};

pub fn main() {
    alfrusco::handle();

    let mut response = Response::new();
    response.skip_knowledge(true);
    response.items(vec![
        Item::new("First Option").subtitle("First Subtitle"),
        Item::new("Option 2").subtitle("Second Subtitle"),
        Item::new("Three").subtitle("3"),
    ]);
    match response.write(std::io::stdout()) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error writing response: {}", e);
            std::process::exit(1);
        }
    }
}
