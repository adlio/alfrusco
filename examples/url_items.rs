use std::time::Duration;

use alfrusco::{Error, URLItem, Workflow};

pub fn main() {
    env_logger::init();
    alfrusco::Workflow::from_env().unwrap().run(run);
}

pub fn run(wf: &mut Workflow) -> Result<(), Error> {
    let _ = wf.response.skip_knowledge(true);
    let _ = wf.response.cache(Duration::from_secs(60), true);
    wf.response.append_items(vec![
        URLItem::new("DuckDuckGo", "https://www.duckduckgo.com").into(),
        URLItem::new("Google", "https://www.google.com").into(),
    ]);
    Ok(())
}
