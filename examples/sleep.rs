use std::{process::Command, time::Duration};

use alfrusco::{URLItem, Workflow, WorkflowError};

pub fn main() {
    Workflow::from_env().unwrap().run(run);
}

pub fn run(wf: &mut Workflow) -> Result<(), WorkflowError> {
    let _ = wf.response.skip_knowledge(true);
    let _ = wf.response.rerun(Duration::from_millis(500));

    let mut cmd = Command::new("/bin/sleep");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.arg("5");

    wf.run_in_background("sleep", Duration::from_secs(6), cmd);

    wf.response
        .append_items(vec![URLItem::new("Google", "https://www.google.com").into()]);
    Ok(())
}
