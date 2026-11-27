use std::collections::HashMap;
use std::io;
use std::time::Duration;

use serde::Serialize;

use crate::response::duration_as_seconds;
use crate::Result;

/// The JSON a script writes to populate Alfred's Text View.
///
/// ```
/// use std::time::Duration;
/// use alfrusco::{ScrollMode, TextView, TextViewMode};
///
/// let mut view = TextView::new("Fruits are the seed-bearing structures in flowering plants.");
/// view.footer("Anatomy of fruits")
///     .mode(TextViewMode::Append)
///     .scroll(ScrollMode::End)
///     .rerun(Duration::from_millis(500))
///     .var("fruit", "banana");
/// view.write(std::io::stdout()).unwrap();
/// ```
///
/// See <https://www.alfredapp.com/help/workflows/user-interface/text/json/>
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TextView {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    variables: HashMap<String, String>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "duration_as_seconds"
    )]
    rerun: Option<Duration>,

    response: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,

    #[serde(rename = "actionoutput", skip_serializing_if = "Option::is_none")]
    action_output: Option<bool>,

    #[serde(skip_serializing_if = "Behaviour::is_empty")]
    behaviour: Behaviour,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct Behaviour {
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<TextViewMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scroll: Option<ScrollMode>,
    #[serde(rename = "inputfield", skip_serializing_if = "Option::is_none")]
    input_field: Option<InputFieldMode>,
}

impl Behaviour {
    fn is_empty(&self) -> bool {
        self.response.is_none() && self.scroll.is_none() && self.input_field.is_none()
    }
}

/// How a Text View rerun changes the text already shown.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextViewMode {
    /// Replace the whole contents. This is Alfred's default.
    Replace,
    /// Add the response to the bottom.
    Append,
    /// Add the response to the top.
    Prepend,
    /// Replace only the previous response.
    ReplaceLast,
}

/// Where the Text View scrolls when new text arrives.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollMode {
    /// Scroll to the start of the new response. This is Alfred's default.
    Auto,
    /// Scroll to the start of the view.
    Start,
    /// Scroll to the end of the view.
    End,
}

/// What happens to the Text View's input field after the user actions it.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputFieldMode {
    /// Erase the input. This is Alfred's default.
    Clear,
    /// Select the input text.
    Select,
}

impl TextView {
    /// A Text View showing `response`. Alfred renders it as Markdown or
    /// plain text, depending on the Text View object's settings.
    pub fn new(response: impl Into<String>) -> Self {
        TextView {
            response: response.into(),
            ..Self::default()
        }
    }

    /// Replaces the text to show.
    pub fn response(&mut self, response: impl Into<String>) -> &mut Self {
        self.response = response.into();
        self
    }

    /// Text shown in the view's footer.
    pub fn footer(&mut self, footer: impl Into<String>) -> &mut Self {
        self.footer = Some(footer.into());
        self
    }

    /// When true, the view closes and passes `response` to the next object.
    pub fn action_output(&mut self, action_output: bool) -> &mut Self {
        self.action_output = Some(action_output);
        self
    }

    /// How this response changes the text already shown.
    pub fn mode(&mut self, mode: TextViewMode) -> &mut Self {
        self.behaviour.response = Some(mode);
        self
    }

    /// Where the view scrolls when this response arrives.
    pub fn scroll(&mut self, scroll: ScrollMode) -> &mut Self {
        self.behaviour.scroll = Some(scroll);
        self
    }

    /// What happens to the input field after the user actions it.
    pub fn input_field(&mut self, input_field: InputFieldMode) -> &mut Self {
        self.behaviour.input_field = Some(input_field);
        self
    }

    /// Reruns the script after `interval` while the view stays open. Alfred
    /// accepts 0.1 to 5 seconds.
    pub fn rerun(&mut self, interval: Duration) -> &mut Self {
        self.rerun = Some(interval);
        self
    }

    /// Sets a session variable, passed out to the workflow stream and back
    /// into the script on rerun.
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
    fn test_minimal_text_view() {
        let view = TextView::new("Hello");
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            json!({"response": "Hello"})
        );
    }

    #[test]
    fn test_full_text_view() {
        let mut view = TextView::new("Fruits are seeds.");
        view.footer("Anatomy")
            .action_output(false)
            .mode(TextViewMode::Append)
            .scroll(ScrollMode::End)
            .input_field(InputFieldMode::Select)
            .rerun(Duration::from_millis(500))
            .var("fruit", "banana");
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            json!({
                "variables": {"fruit": "banana"},
                "rerun": 0.5,
                "response": "Fruits are seeds.",
                "footer": "Anatomy",
                "actionoutput": false,
                "behaviour": {"response": "append", "scroll": "end", "inputfield": "select"}
            })
        );
    }

    #[test]
    fn test_mode_values() {
        for (mode, expected) in [
            (TextViewMode::Replace, "replace"),
            (TextViewMode::Append, "append"),
            (TextViewMode::Prepend, "prepend"),
            (TextViewMode::ReplaceLast, "replacelast"),
        ] {
            assert_eq!(serde_json::to_value(mode).unwrap(), json!(expected));
        }
        for (scroll, expected) in [
            (ScrollMode::Auto, "auto"),
            (ScrollMode::Start, "start"),
            (ScrollMode::End, "end"),
        ] {
            assert_eq!(serde_json::to_value(scroll).unwrap(), json!(expected));
        }
        for (input_field, expected) in [
            (InputFieldMode::Clear, "clear"),
            (InputFieldMode::Select, "select"),
        ] {
            assert_eq!(serde_json::to_value(input_field).unwrap(), json!(expected));
        }
    }

    #[test]
    fn test_response_replaces_text_and_partial_behaviour() {
        let mut view = TextView::new("first");
        view.response("second")
            .action_output(true)
            .scroll(ScrollMode::Start);
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            json!({
                "response": "second",
                "actionoutput": true,
                "behaviour": {"scroll": "start"}
            })
        );
    }

    #[test]
    fn test_write() {
        let mut buffer = Vec::new();
        TextView::new("Hi").write(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), r#"{"response":"Hi"}"#);
    }
}
