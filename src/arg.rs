use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum Arg {
    One(String),
    Many(Vec<String>),
}

impl From<&str> for Arg {
    fn from(value: &str) -> Self {
        Arg::One(value.to_string())
    }
}

impl From<String> for Arg {
    fn from(value: String) -> Self {
        Arg::One(value)
    }
}

impl From<&String> for Arg {
    fn from(value: &String) -> Self {
        Arg::One(value.clone())
    }
}

impl<S: Into<String>> From<Vec<S>> for Arg {
    fn from(values: Vec<S>) -> Self {
        Arg::Many(values.into_iter().map(Into::into).collect())
    }
}

impl<S: Into<String>, const N: usize> From<[S; N]> for Arg {
    fn from(values: [S; N]) -> Self {
        Arg::Many(values.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Arg;
    use crate::Item;

    #[test]
    fn test_from_conversions() {
        let owned = String::from("hello");
        for one in [
            Arg::from("hello"),
            Arg::from(&owned),
            Arg::from(owned.clone()),
        ] {
            assert_eq!(serde_json::to_value(one).unwrap(), json!("hello"));
        }
        for many in [
            Arg::from(vec!["hello", "world"]),
            Arg::from(["hello", "world"]),
            Arg::from(vec![String::from("hello"), String::from("world")]),
        ] {
            assert_eq!(
                serde_json::to_value(many).unwrap(),
                json!(["hello", "world"])
            );
        }
        // A one-element collection stays an array.
        assert_eq!(
            serde_json::to_value(Arg::from(["hello"])).unwrap(),
            json!(["hello"])
        );
    }

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
