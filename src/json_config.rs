use std::collections::HashMap;
use std::io;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{Arg, Result};

/// The `alfredworkflow` JSON a Run Script action (or a JSON Utility) writes
/// to set the next object's argument, override its configuration and set
/// workflow variables.
///
/// ```
/// use alfrusco::JsonConfig;
///
/// let mut output = JsonConfig::new();
/// output
///     .arg("https://www.alfredapp.com")
///     .config("browser", "com.apple.Safari")
///     .var("fruit", "banana");
/// output.write(std::io::stdout()).unwrap();
/// ```
///
/// See <https://www.alfredapp.com/help/workflows/utilities/json/json/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsonConfig {
    arg: Option<Arg>,
    config: Map<String, Value>,
    variables: HashMap<String, String>,
}

#[derive(Serialize)]
struct Envelope<'a> {
    alfredworkflow: Body<'a>,
}

#[derive(Serialize)]
struct Body<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<&'a Arg>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    config: &'a Map<String, Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    variables: &'a HashMap<String, String>,
}

impl Serialize for JsonConfig {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        Envelope {
            alfredworkflow: Body {
                arg: self.arg.as_ref(),
                config: &self.config,
                variables: &self.variables,
            },
        }
        .serialize(s)
    }
}

impl JsonConfig {
    /// An output that sets nothing. Note that Alfred clears the query when
    /// `arg` is left unset; see [`JsonConfig::arg`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The argument passed to the next object: a string, or a `Vec` or array
    /// for several. Leaving it unset clears the query, so a script that only
    /// wants to set `config` or variables should still pass along the value
    /// the next object needs, such as its own input.
    pub fn arg(&mut self, arg: impl Into<Arg>) -> &mut Self {
        self.arg = Some(arg.into());
        self
    }

    /// Overrides one configuration field of the connected objects. Only the
    /// fields you set are overridden. To find an object's field names,
    /// right-click it on Alfred's canvas and choose Copy Configuration.
    pub fn config(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Sets a workflow variable for the rest of the workflow stream.
    pub fn var(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Writes the JSON to `writer`.
    pub fn write<W: io::Write>(&self, writer: W) -> Result<()> {
        Ok(serde_json::to_writer(writer, self)?)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(
            serde_json::to_value(JsonConfig::new()).unwrap(),
            json!({"alfredworkflow": {}})
        );
    }

    #[test]
    fn test_full() {
        let mut output = JsonConfig::new();
        output
            .arg(["apples", "oranges"])
            .config("browser", "com.apple.Safari")
            .config("skipqueryencode", false)
            .var("fruit", "banana");
        assert_eq!(
            serde_json::to_value(&output).unwrap(),
            json!({
                "alfredworkflow": {
                    "arg": ["apples", "oranges"],
                    "config": {"browser": "com.apple.Safari", "skipqueryencode": false},
                    "variables": {"fruit": "banana"}
                }
            })
        );
    }

    #[test]
    fn test_write() {
        let mut buffer = Vec::new();
        let mut output = JsonConfig::new();
        output.arg("x");
        output.write(&mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            r#"{"alfredworkflow":{"arg":"x"}}"#
        );
    }
}
