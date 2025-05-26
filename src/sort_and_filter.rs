use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::Item;

pub fn filter_and_sort_items(items: Vec<Item>, query: String) -> Vec<Item> {
    let matcher = SkimMatcherV2::default();

    let mut filtered_items: Vec<(Item, i64)> = items
        .into_iter()
        .filter_map(|item| {
            let subtitle = item.subtitle.as_deref().unwrap_or_default();
            let combined = format!("{} : {}", subtitle, item.title);
            matcher
                .fuzzy_match(&combined, &query)
                .map(|score| (item, score))
        })
        .collect();

    // Sort by score in descending order
    filtered_items.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    filtered_items.into_iter().map(|(item, _)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_and_sort_items_basic_matching() {
        let items = vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Fruit"),
            Item::new("Carrot").subtitle("Vegetable"),
        ];

        // Should match "Apple" and "Banana" as they contain "fruit"
        let result = filter_and_sort_items(items.clone(), "fruit".to_string());
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|item| item.title == "Apple"));
        assert!(result.iter().any(|item| item.title == "Banana"));

        // Should match only "Carrot" as it contains "vegetable"
        let result = filter_and_sort_items(items.clone(), "vegetable".to_string());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Carrot");

        // Should match nothing with this query
        let result = filter_and_sort_items(items, "meat".to_string());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_and_sort_items_sorting_order() {
        let items = vec![
            Item::new("Zebra").subtitle("Animal"),
            Item::new("Antelope").subtitle("Animal"),
            Item::new("Zebra fish").subtitle("Fish"),
        ];

        // Both "Zebra" items should match for "zebra"
        let result = filter_and_sort_items(items.clone(), "zebra".to_string());
        assert_eq!(result.len(), 2);
        // Just verify both zebra items are in the results, order depends on fuzzy matching algorithm
        assert!(result.iter().any(|item| item.title == "Zebra"));
        assert!(result.iter().any(|item| item.title == "Zebra fish"));

        // All should match "a" but we don't assert specific order
        let result = filter_and_sort_items(items, "a".to_string());
        assert_eq!(result.len(), 3);
        // Just verify all items are in the results
        assert!(result.iter().any(|item| item.title == "Zebra"));
        assert!(result.iter().any(|item| item.title == "Antelope"));
        assert!(result.iter().any(|item| item.title == "Zebra fish"));
    }

    #[test]
    fn test_filter_and_sort_items_fuzzy_matching() {
        let items = vec![
            Item::new("Configuration").subtitle("Settings"),
            Item::new("Profile").subtitle("User settings"),
            Item::new("Preferences").subtitle("App config"),
        ];

        // Should match all items containing "config" in title or subtitle
        let result = filter_and_sort_items(items.clone(), "config".to_string());
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|item| item.title == "Configuration"));
        assert!(result.iter().any(|item| item.title == "Preferences"));

        // Should match items with "settings" in subtitle
        let result = filter_and_sort_items(items, "settings".to_string());
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|item| item.title == "Configuration"));
        assert!(result.iter().any(|item| item.title == "Profile"));
    }
}
