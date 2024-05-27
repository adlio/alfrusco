use std::io;
use std::time::Duration;

use anyhow::Result;
use serde::{Serialize, Serializer};

use crate::Item;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Response {
    /// Interval in seconds to wait before re-running the script filter
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "duration_as_seconds"
    )]
    rerun: Option<Duration>,

    /// If true, Alfred will not learn from the user's selection
    #[serde(rename = "skipknowledge", skip_serializing_if = "Option::is_none")]
    skip_knowledge: Option<bool>,

    /// The items to display in Alfred's output
    items: Vec<Item>,
}

impl Response {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_items(items: Vec<Item>) -> Self {
        Self {
            items,
            ..Self::default()
        }
    }

    pub fn rerun(&mut self, duration: Duration) -> &mut Self {
        self.rerun = Some(duration);
        self
    }

    pub fn items(&mut self, items: Vec<Item>) -> &mut Self {
        self.items.extend(items);
        self
    }

    /// Writes the Alfred response to the provided writer.
    pub fn write<W: io::Write>(&self, writer: W) -> serde_json::Result<()> {
        serde_json::to_writer(writer, self)
    }
}

fn duration_as_seconds<S>(duration: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match duration {
        Some(duration) => s.serialize_f32(duration.as_secs_f32()),
        None => unreachable!(),
    }
}
