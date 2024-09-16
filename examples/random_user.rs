use alfrusco::{AsyncRunnable, Item, Workflow, WorkflowConfig, WorkflowError, WorkflowResult};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
struct RandomUserWorkflow {
    pub keyword: Vec<String>,
    #[arg(short, long, env)]
    pub name: Option<String>,
}

#[tokio::main]
pub async fn main() {
    let command = RandomUserWorkflow::parse();
    let config = WorkflowConfig::from_env().unwrap();
    Workflow::execute_async(config, command, &mut std::io::stdout()).await;
}

#[async_trait::async_trait]
impl AsyncRunnable for RandomUserWorkflow {
    async fn run_async(self, wf: &mut Workflow) -> WorkflowResult {
        match self.name {
            Some(name) => {
                wf.append_item(Item::new(format!("NAME DEFINED AS: '{}'", name)));
                return Ok(());
            }
            None => {}
        }

        let query = self.keyword.join(" ");
        wf.set_filter_keyword(query.clone());

        let url = "https://randomuser.me/api/?inc=gender,name&results=50&seed=alfrusco";
        let response = reqwest::get(url)
            .await
            .map_err(|e| WorkflowError::new(e.to_string()))?;
        let response: RandomUserResponse = response
            .json()
            .await
            .map_err(|e| WorkflowError::new(e.to_string()))?;
        wf.append_items(
            response
                .results
                .into_iter()
                .map(|r| {
                    let title = format!("{} {} {}", r.name.title, r.name.first, r.name.last);
                    Item::new(&title)
                        .valid(false)
                        .autocomplete("workflow:nonsense")
                        .var("NAME", title)
                })
                .collect(),
        );
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomUserResponse {
    pub results: Vec<Result>,
    pub info: serde_json::Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub name: RandomUserName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomUserName {
    pub title: String,
    pub first: String,
    pub last: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_random_user_response() {
        let config = WorkflowConfig::for_testing().unwrap();
        let workflow = RandomUserWorkflow {
            keyword: vec![],
            name: None,
        };
        let mut buffer = Vec::new();
        alfrusco::Workflow::execute_async(config, workflow, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"title\":\"Mr Fletcher Hall\""));
    }
}
