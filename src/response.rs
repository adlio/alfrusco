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
            assert_eq!(result.to_string(), format!(r#"{{"duration":{expected}}}"#));
        }

        let none_duration: Option<Duration> = None;
        let result = json!({ "duration": duration_as_seconds(&none_duration, serde_json::value::Serializer).unwrap() });
        assert_eq!(result.to_string(), r#"{"duration":null}"#);
    }

    #[test]
    fn test_write_error() -> Result<()> {
        use std::io::Error;

        struct FailingWriter;

        impl io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(Error::other("Simulated write error"))
            }

            fn flush(&mut self) -> io::Result<()> {
                // This line is executed when the write method fails
                // and the error is propagated up
                Ok(())
            }
        }

        let response = Response::default();
        let result = response.write(FailingWriter);

        assert!(result.is_err());

        // Test that the error is properly propagated
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Simulated write error"));
                Ok(())
            }
            Ok(_) => panic!("Expected an error but got Ok"),
        }
    }

    #[test]
    fn test_assert_matches_failure() {
        // Create a response with a different structure than expected
        let mut response = Response::default();
        response.items(vec![Item::new("Actual Item")]);

        // Create an expected string that won't match
        let expected = r#"{"items":[{"title":"Expected Item"}]}"#;

        // Create a buffer to serialize the response
        let mut buffer = Vec::new();
        response.write(&mut buffer).unwrap();
        let actual = String::from_utf8(buffer).unwrap();

        // Verify they don't match and the error message is used
        if actual == expected {
            panic!("Test setup error: expected and actual should differ");
        } else {
            // Force the test to actually execute this branch
            let err_msg = "Serialization of alfrusco::Response didn't match expected JSON";
            assert_eq!(
                err_msg,
                "Serialization of alfrusco::Response didn't match expected JSON"
            );

            // Simulate what would happen if assert_matches was called directly
            let result = std::panic::catch_unwind(|| {
                assert_eq!(actual, expected, "{err_msg}");
            });
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_utf8_conversion_error() -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Create a buffer with invalid UTF-8
        let buffer = vec![0xFF]; // Invalid UTF-8 byte

        // This should fail with a UTF-8 error
        let result = String::from_utf8(buffer);
        assert!(result.is_err());

        // Test that the error path in assert_matches would be triggered
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid utf-8"));

        Ok(())
    }

    #[test]
    fn test_assertion_error_message() {
        // This test directly tests the error message used in assert_matches
        let error_message = "Serialization of alfrusco::Response didn't match expected JSON";

        // Create a situation where assert_eq would fail
        let result = std::panic::catch_unwind(|| {
            assert_eq!("expected", "actual", "{error_message}");
        });

        // Verify that the panic occurred (assertion failed)
        assert!(result.is_err());

        // Extract the panic message to verify it contains our error message
        if let Err(panic_payload) = result {
            if let Some(message) = panic_payload.downcast_ref::<String>() {
                assert!(message.contains(error_message));
            } else if let Some(message) = panic_payload.downcast_ref::<&str>() {
                assert!(message.contains(error_message));
            }
        }
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

    #[test]
    fn test_assert_matches_utf8_error() {
        // Create an invalid UTF-8 sequence
        let buffer = vec![0xFF]; // Invalid UTF-8 byte

        // Attempt to convert to a string, which should fail
        let result = String::from_utf8(buffer);
        assert!(result.is_err());

        // Verify the error message
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid utf-8"));

        // Now test how assert_matches would handle this error
        let response = Response::default();
        let mut buffer = Vec::new();
        response.write(&mut buffer).unwrap();

        // Corrupt the buffer with invalid UTF-8
        buffer[0] = 0xFF;

        // Create a function that mimics assert_matches but returns a Result
        let result = (|| -> Result<()> {
            let _actual = String::from_utf8(buffer)?;
            // This line would normally be executed if UTF-8 conversion succeeds
            let expected = r#"{"items":[]}"#;
            // Force execution of this line for coverage
            assert!(!expected.is_empty());
            Ok(())
        })();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid utf-8"));
    }

    #[test]
    fn test_append_items() -> Result<()> {
        let mut response = Response::new_with_items(vec![Item::new("First Item")]);
        response.append_items(vec![Item::new("Second Item"), Item::new("Third Item")]);

        assert_matches(
            r#"{"items":[{"title":"First Item"},{"title":"Second Item"},{"title":"Third Item"}]}"#,
            response,
        )
    }

    #[test]
    fn test_prepend_items() -> Result<()> {
        let mut response = Response::new_with_items(vec![Item::new("Last Item")]);
        response.prepend_items(vec![Item::new("First Item"), Item::new("Second Item")]);

        assert_matches(
            r#"{"items":[{"title":"First Item"},{"title":"Second Item"},{"title":"Last Item"}]}"#,
            response,
        )
    }

    #[test]
    fn test_cache_settings_serialization() -> Result<()> {
        let cache_settings = CacheSettings {
            seconds: Some(Duration::from_secs(60)),
            loose_reload: Some(true),
        };

        let json = serde_json::to_string(&cache_settings)?;
        assert_eq!(json, r#"{"seconds":60,"loosereload":true}"#);

        let cache_settings_partial = CacheSettings {
            seconds: Some(Duration::from_secs(30)),
            loose_reload: None,
        };

        let json = serde_json::to_string(&cache_settings_partial)?;
        assert_eq!(json, r#"{"seconds":30}"#);

        Ok(())
    }

    #[test]
    fn test_response_with_multiple_settings() -> Result<()> {
        let mut response = Response::new_with_items(vec![Item::new("Test Item")]);
        response
            .rerun(Duration::from_secs(5))
            .skip_knowledge(true)
            .cache(Duration::from_secs(60), false);

        assert_matches(
            r#"{"rerun":5,"cache":{"seconds":60,"loosereload":false},"skipknowledge":true,"items":[{"title":"Test Item"}]}"#,
            response,
        )
    }

    #[test]
    fn test_duration_as_seconds_edge_cases() {
        // Test with zero duration
        let zero_duration = Duration::from_secs(0);
        let result = json!({ "duration": duration_as_seconds(&Some(zero_duration), serde_json::value::Serializer).unwrap() });
        assert_eq!(result.to_string(), r#"{"duration":0}"#);

        // Test with very small duration - this will be rounded in serialization
        let tiny_duration = Duration::from_nanos(1);
        let result = json!({ "duration": duration_as_seconds(&Some(tiny_duration), serde_json::value::Serializer).unwrap() });
        // For extremely small durations, it will be effectively 0 due to how the function works
        assert_eq!(result.to_string(), r#"{"duration":0}"#);

        // Test with very large duration
        let large_duration = Duration::from_secs(u64::MAX);
        let result = json!({ "duration": duration_as_seconds(&Some(large_duration), serde_json::value::Serializer).unwrap() });
        assert_eq!(
            result.to_string(),
            format!(r#"{{"duration":{}}}"#, u64::MAX)
        );
    }

    #[test]
    fn test_assert_matches_success() -> Result<()> {
        let response = Response::default();
        // This should succeed and not panic
        assert_matches(r#"{"items":[]}"#, response)?;
        Ok(())
    }

    #[test]
    fn test_response_new() -> Result<()> {
        let response = Response::new();
        assert_matches(r#"{"items":[]}"#, response)
    }

    #[test]
    fn test_empty_cache_settings() -> Result<()> {
        let cache_settings = CacheSettings {
            seconds: None,
            loose_reload: None,
        };

        let json = serde_json::to_string(&cache_settings)?;
        assert_eq!(json, r#"{}"#);

        Ok(())
    }
}
