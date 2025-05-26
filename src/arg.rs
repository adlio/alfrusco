use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum Arg {
    One(String),
    Many(Vec<String>),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Item;

    #[test]
    fn test_arg_one() {
        let item = Item::new("Hello").arg("hello");
        let json = serde_json::to_value(item.arg).unwrap();
        let expected = json!("hello");
        assert_eq!(json, expected);
    }

    #[test]
    fn test_arg_many() {
        let item = Item::new("Array").args(vec!["hello", "world"]);
        let json = serde_json::to_value(item.arg).unwrap();
        let expected = json!(["hello", "world"]);
        assert_eq!(json, expected);
    }
}
