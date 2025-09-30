use alfrusco::{config, Runnable, URLItem, Workflow};
use clap::Parser;

#[derive(Parser)]
struct URLItemsWithCustomArgsWorkflow {}

pub fn main() {
    env_logger::init();
    let command = URLItemsWithCustomArgsWorkflow {};
    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl Runnable for URLItemsWithCustomArgsWorkflow {
    type Error = alfrusco::Error;
    
    fn run(self, wf: &mut Workflow) -> Result<(), Self::Error> {
        wf.skip_knowledge(true);
        
        // Regular URLItem - arg defaults to URL
        let regular_url_item = URLItem::new("Rust Documentation", "https://doc.rust-lang.org/");
        
        // URLItem with custom arg for Alfred workflow navigation
        let search_item = URLItem::new("Advanced Search", "https://example.com/search")
            .subtitle("Search with custom filters")
            .arg("workflow:search:advanced");
            
        // URLItem that opens a specific Alfred workflow
        let workflow_item = URLItem::new("Open Settings", "https://myapp.com/settings")
            .subtitle("Configure application settings")
            .arg("workflow:settings:main");
        
        wf.append_items(vec![
            regular_url_item.into(),
            search_item.into(), 
            workflow_item.into(),
        ]);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_items_with_custom_args_workflow() {
        let command = URLItemsWithCustomArgsWorkflow {};
        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute(&config::TestingProvider(dir), command, &mut buffer);
        let output = String::from_utf8(buffer).unwrap();
        
        // Check that regular URL item uses URL as arg
        assert!(output.contains("\"arg\":\"https://doc.rust-lang.org/\""));
        
        // Check that custom arg items use their custom args
        assert!(output.contains("\"arg\":\"workflow:search:advanced\""));
        assert!(output.contains("\"arg\":\"workflow:settings:main\""));
        
        // Check that custom subtitles are preserved
        assert!(output.contains("\"subtitle\":\"Search with custom filters\""));
        assert!(output.contains("\"subtitle\":\"Configure application settings\""));
    }
}