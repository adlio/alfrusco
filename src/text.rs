use serde::Serialize;

/// Text defines the two text options (copy and largetext) for an Alfred
/// Item.
///
/// The copy property is the text that is copied to the clipboard when
/// the user pressed CMD-C. The largetype property is the content displayed
/// when the user presses CMD-L.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) copy: Option<String>,

    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    pub(crate) large_type: Option<String>,
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::Item;

    #[test]
    fn test_copy() {
        let item = Item::new("Item").copy_text("will be copied");
        let json = serde_json::to_value(&item.text).unwrap();
        let expected = json!({"copy": "will be copied"});
        assert_eq!(json, expected);
    }

    #[test]
    fn test_large_type() {
        let item = Item::new("Item").large_type_text("how big am I");
        let json = serde_json::to_value(&item.text).unwrap();
        let expected = json!({"largetype": "how big am I"});
        assert_eq!(json, expected);
    }
}
