use std::time::Duration;

use alfrusco::{URLItem, Workflow};

pub fn main() {
    let config = alfrusco::WorkflowConfig::for_testing().unwrap();
    alfrusco::Workflow::run(config, run);
}

pub fn run(wf: &mut Workflow) -> Result<(), Box<dyn std::error::Error>> {
    wf.run_in_background("sleep", Duration::from_secs(5), "sleep 5".into())?;
    let _ = wf.response.skip_knowledge(true);
    let _ = wf.response.cache(Duration::from_secs(60), true);
    wf.response.append_items(vec![
        URLItem::new("DuckDuckGo", "https://www.duckduckgo.com").into(),
        URLItem::new("Google", "https://www.google.com").into(),
    ]);
    Ok(())
}
