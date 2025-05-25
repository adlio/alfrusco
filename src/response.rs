use std::io;
use std::time::Duration;

use serde::{Serialize, Serializer};

use crate::{Item, Result};

/// Represents the contents of a complete Alfred response to an execution.
///
/// It consists of the `.items` to display in Alfred's UI and optional
/// configuration settings to control re-running the workflow, caching,
/// and disabling Alfred's learning of the response the user selected
/// (skip_knowledge).
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
    pub(crate) skip_knowledge: Option<bool>,

    /// The items to display in Alfred's output
    pub(crate) items: Vec<Item>,
}

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
    // Creates a new, empty Alfred response with an empty vec of Items.
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

    /// Replaces the existing items in the response with the provided ones.
    pub fn items(&mut self, items: Vec<Item>) -> &mut Self {
        self.items = items;
        self
    }

    /// Appends the provided items to the end of the existing items in the reponse.
    pub fn append_items(&mut self, items: Vec<Item>) {
        self.items.extend(items);
    }

    /// Prepends the provided items to the beginning of the existing items in the
    /// response.
    pub fn prepend_items(&mut self, items: Vec<Item>) {
        self.items.splice(0..0, items);
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
        Some(duration) => {
            let secs = duration.as_secs();
            let subsec_millis = duration.subsec_millis();

            if subsec_millis == 0 {
                s.serialize_u64(secs)
            } else {
                let millis = secs * 1000 + u64::from(subsec_millis);
                let seconds = millis as f64 / 1000.0;
                s.serialize_f64(seconds)
            }
        }
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use super::*;

    #[test]
    fn test_empty_response() -> Result<()> {
        let response = Response::default();
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
        let mut response = Response::default();
        response.rerun(Duration::from_secs(5));
        assert_matches(r#"{"rerun":5,"items":[]}"#, response)
    }

    #[test]
    fn test_skip_knowledge() -> Result<()> {
        let mut response = Response::default();
        response.skip_knowledge(true);
        assert_matches(r#"{"skipknowledge":true,"items":[]}"#, response)
    }

    #[test]
    fn test_cache() -> Result<()> {
        let mut response = Response::default();
        response.cache(Duration::from_secs(10800), true);
        assert_matches(
            r#"{"cache":{"seconds":10800,"loosereload":true},"items":[]}"#,
            response,
        )
    }

    #[test]
    fn test_simple_item() -> Result<()> {
        let mut response = Response::default();
        response.items(vec![Item::new("Simple Title")]);
        assert_matches(r#"{"items":[{"title":"Simple Title"}]}"#, response)
    }

    #[test]
    fn test_duration_as_seconds_serialization() {
        let cases = [
            (Duration::from_millis(400), "0.4"),
            (Duration::from_millis(432), "0.432"),
            (Duration::from_secs(2), "2"),
            (Duration::from_secs(60), "60"),
            (Duration::from_secs(300), "300"),
            (Duration::from_millis(2500), "2.5"),
        ];

        for (duration, expected) in cases {
            let result = json!({ "duration": duration_as_seconds(&Some(duration), serde_json::value::Serializer).unwrap() });
            assert_eq!(
                result.to_string(),
                format!(r#"{{"duration":{expected}}}"#)
            );
        }

        let none_duration: Option<Duration> = None;
        let result = json!({ "duration": duration_as_seconds(&none_duration, serde_json::value::Serializer).unwrap() });
        assert_eq!(result.to_string(), r#"{"duration":null}"#);
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
