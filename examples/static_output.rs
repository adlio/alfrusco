use alfrusco::{Item, Workflow};

pub fn main() {
    let config = alfrusco::WorkflowConfig::for_testing().unwrap();
    alfrusco::Workflow::run(config, run);
}

pub fn run(wf: &mut Workflow) -> Result<(), Box<dyn std::error::Error>> {
    let _ = &wf.response.skip_knowledge(true);
    &wf.response.append_items(vec![
        Item::new("First Option").subtitle("First Subtitle"),
        Item::new("Option 2").subtitle("Second Subtitle"),
        Item::new("Three").subtitle("3"),
    ]);

    match &wf.response.write(std::io::stdout()) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error writing response: {}", e);
            std::process::exit(1);
        }
    }
}
