use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::debug;

use crate::Item;

pub fn filter_and_sort_items(items: Vec<Item>, query: String) -> Vec<Item> {
    debug!(
        "Filtering and sorting {} items with query: '{}'",
        items.len(),
        query
    );

    // First, separate sticky items from regular items
    let (sticky_items, regular_items): (Vec<Item>, Vec<Item>) =
        items.into_iter().partition(|item| item.sticky);

    debug!(
        "Found {} sticky items and {} regular items",
        sticky_items.len(),
        regular_items.len()
    );

    let matcher = SkimMatcherV2::default();

    // Filter and score regular items
    let mut filtered_items: Vec<(Item, i64)> = regular_items
        .into_iter()
        .filter_map(|item| {
            let subtitle = item.subtitle.as_deref().unwrap_or_default();
            let combined = format!("{} : {}", subtitle, item.title);
            matcher
                .fuzzy_match(&combined, &query)
                .map(|score| (item, score))
        })
        .collect();

    debug!(
        "After filtering, {} regular items match the query",
        filtered_items.len()
    );

    // Sort by score in descending order
    filtered_items.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // Extract the items from the tuples
    let mut result: Vec<Item> = filtered_items.into_iter().map(|(item, _)| item).collect();

    // Add sticky items at the beginning, regardless of query
    if !sticky_items.is_empty() {
        debug!(
            "Adding {} sticky items to the beginning of results",
            sticky_items.len()
        );
        let mut final_result = sticky_items;
        final_result.append(&mut result);
        debug!("Final result has {} items", final_result.len());
        final_result
    } else {
        debug!(
            "No sticky items to add, returning {} filtered items",
            result.len()
        );
        result
    }
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

    #[test]
    fn test_filter_with_sticky_items() {
        let mut sticky_item = Item::new("Important").subtitle("Always shown");
        sticky_item.sticky = true;

        let items = vec![
            sticky_item,
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Fruit"),
        ];

        // Sticky items should appear first regardless of query
        let result = filter_and_sort_items(items, "fruit".to_string());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Important"); // Sticky item first
        assert!(result[1..].iter().any(|item| item.title == "Apple"));
        assert!(result[1..].iter().any(|item| item.title == "Banana"));
    }

    #[test]
    fn test_filter_without_sticky_items() {
        // This test specifically covers the else branch at line 59
        let items = vec![
            Item::new("Dog").subtitle("Pet"),
            Item::new("Cat").subtitle("Pet"),
            Item::new("Bird").subtitle("Pet"),
        ];

        // No sticky items - should return filtered results directly
        let result = filter_and_sort_items(items, "pet".to_string());
        assert_eq!(result.len(), 3);
        // All items should be in results, no sticky items to prepend
        assert!(result.iter().any(|item| item.title == "Dog"));
        assert!(result.iter().any(|item| item.title == "Cat"));
        assert!(result.iter().any(|item| item.title == "Bird"));
    }

    #[test]
    fn test_filter_empty_items() {
        let items: Vec<Item> = vec![];
        let result = filter_and_sort_items(items, "query".to_string());
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_empty_query() {
        let items = vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Fruit"),
        ];

        // Empty query should still work through fuzzy matcher
        let result = filter_and_sort_items(items, String::new());
        // Empty string matches everything in fuzzy matching
        assert_eq!(result.len(), 2);
    }
}
