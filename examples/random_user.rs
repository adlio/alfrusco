use alfrusco::{AsyncRunnable, Item, Workflow, WorkflowConfig, WorkflowError};
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
    type Error = RandomUserError;

    async fn run_async(self, wf: &mut Workflow) -> Result<(), RandomUserError> {
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
        let response = reqwest::get(url).await?;
        let response: RandomUserResponse = response.json().await?;
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
    pub results: Vec<RandomUser>,
    pub info: serde_json::Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomUser {
    pub name: RandomUserName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomUserName {
    pub title: String,
    pub first: String,
    pub last: String,
}

#[derive(Debug)]
pub enum RandomUserError {
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
}

impl From<reqwest::Error> for RandomUserError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for RandomUserError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl WorkflowError for RandomUserError {}

impl std::fmt::Display for RandomUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomUserError::Reqwest(e) => write!(f, "Reqwest error: {}", e),
            RandomUserError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for RandomUserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RandomUserError::Reqwest(e) => Some(e),
            RandomUserError::Json(e) => Some(e),
        }
    }
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
