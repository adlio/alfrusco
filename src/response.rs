use crate::Item;
use crate::Result;
use serde::{Serialize, Serializer};
use std::io;
use std::time::Duration;

/// Represents a complete Alfred response consisting of Items to display in
/// Alfred's UI and optional configuration settings to control re-running
/// the workflow an disabling Alfred's learning of the response the user
/// selected to re-order items on future runs.
///
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Response {
    /// Interval in seconds to wait before re-running the script filter
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "duration_as_seconds"
    )]
    rerun: Option<Duration>,

    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<CacheSettings>,

    /// If true, Alfred will not learn from the user's selection
    #[serde(rename = "skipknowledge", skip_serializing_if = "Option::is_none")]
    skip_knowledge: Option<bool>,

    /// The items to display in Alfred's output
    items: Vec<Item>,

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CacheSettings {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "duration_as_seconds"
    )]
    pub seconds: Option<Duration>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "loosereload")]
    pub loose_reload: Option<bool>,
}

impl Response {
    /// Creates a new, empty Alfred response.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new Alfred response with the provided Vec of Items.
    pub fn new_with_items(items: Vec<Item>) -> Self {
        Self {
            items,
            ..Self::default()
        }
    }

    /// Sets the rerun interval for this Alfred response.
    pub fn rerun(&mut self, duration: Duration) -> &mut Self {
        self.rerun = Some(duration);
        self
    }

    /// When set to true, Alfred will not learn from the user's selection.
    pub fn skip_knowledge(&mut self, skip_knowledge: bool) -> &mut Self {
        self.skip_knowledge = Some(skip_knowledge);
        self
    }

    /// Extends the existing vec of Items with the provided Vec of Items.
    /// Enables the Alfred 5.5+ cache feature with the provided cache duration.
    /// If loose_reload is true, Alfred will return the stale results while
    /// waiting for the cache to be updated.
    ///
    pub fn cache(&mut self, duration: Duration, loose_reload: bool) -> &mut Self {
        self.cache = Some(CacheSettings {
            seconds: Some(duration),
            loose_reload: Some(loose_reload),
        });
        self
    }

    pub fn items(&mut self, items: Vec<Item>) -> &mut Self {
        self.items.extend(items);
        self
    }

    /// Writes the Alfred response to the provided writer.
    pub fn write<W: io::Write>(&self, writer: W) -> Result<()> {
        Ok(serde_json::to_writer(writer, self)?)
    }
}

/// Custom serializer for serializing a Duration as a floating point number
/// of seconds (the expected format for Alfred's rerun field).
///
fn duration_as_seconds<S>(duration: &Option<Duration>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match duration {
        Some(duration) => s.serialize_f32(duration.as_secs_f32()),
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_response() -> Result<()> {
        let response = Response::new();
        assert_matches(r#"{"items":[]}"#, response)
    }

    #[test]
    fn test_new_with_items() -> Result<()> {
        let response = Response::new_with_items(vec![
            Item::new("Title"),
            Item::new("Another Title"),
            Item::new("Title Number 3"),
        ]);
        assert_matches(
            r#"{"items":[{"title":"Title"},{"title":"Another Title"},{"title":"Title Number 3"}]}"#,
            response,
        )
    }

    #[test]
    fn test_rerun_serialization() -> Result<()> {
        let mut response = Response::new();
        response.rerun(Duration::from_secs(5));
        assert_matches(r#"{"rerun":5.0,"items":[]}"#, response)
    }

    #[test]
    fn test_skip_knowledge() -> Result<()> {
        let mut response = Response::new();
        response.skip_knowledge(true);
        assert_matches(r#"{"skipknowledge":true,"items":[]}"#, response)
    }

    #[test]
    fn test_cache() -> Result<()> {
        let mut response = Response::new();
        response.cache(Duration::from_secs(10800), true);
        assert_matches(
            r#"{"cache":{"seconds":10800.0,"loosereload":true},"items":[]}"#,
            response,
        )
    }

    #[test]
    fn test_simple_item() -> Result<()> {
        let mut response = Response::new();
        response.items(vec![Item::new("Simple Title")]);
        assert_matches(r#"{"items":[{"title":"Simple Title"}]}"#, response)
    }

    fn assert_matches(expected: &str, response: Response) -> Result<()> {
        let mut buffer = Vec::new();
        response.write(&mut buffer)?;

        let actual = String::from_utf8(buffer)?;
        assert_eq!(
            actual, expected,
            "Serialization of alfrusco::Response didn't match expected JSON"
        );
        Ok(())
    }
}
