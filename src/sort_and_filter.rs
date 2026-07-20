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

    // Separate items into three groups: sticky (pinned to top), bottom-pinned
    // (pinned to bottom), and regular items (fuzzy filtered + sorted).
    let (sticky_items, rest): (Vec<Item>, Vec<Item>) =
        items.into_iter().partition(|item| item.sticky);
    let (bottom_items, regular_items): (Vec<Item>, Vec<Item>) =
        rest.into_iter().partition(|item| item.pin_to_bottom);

    debug!(
        "Found {} sticky items, {} bottom-pinned items, and {} regular items",
        sticky_items.len(),
        bottom_items.len(),
        regular_items.len()
    );

    // Case-insensitive matching: the default "smart case" turns
    // case-sensitive when the query contains any uppercase letter, which
    // makes mixed-case queries (e.g. "Nasa Hq") fail against all-caps
    // candidates (e.g. "NASA HQ"). Launcher-style filtering should never
    // gate on case.
    let matcher = SkimMatcherV2::default().ignore_case();

    // Filter and score regular items, adding boost to the score
    let mut filtered_items: Vec<(Item, i64)> = regular_items
        .into_iter()
        .filter_map(|item| {
            let subtitle = item.subtitle.as_deref().unwrap_or_default();
            let combined = format!("{} : {}", subtitle, item.title);
            let boost = item.boost;
            matcher
                .fuzzy_match(&combined, &query)
                .map(|score| (item, score + boost))
        })
        .collect();

    debug!(
        "After filtering, {} regular items match the query",
        filtered_items.len()
    );

    // Sort by score in descending order
    filtered_items.sort_unstable_by_key(|item| std::cmp::Reverse(item.1));

    // Extract the items from the tuples
    let result: Vec<Item> = filtered_items.into_iter().map(|(item, _)| item).collect();

    // Assemble: sticky items first, filtered regular items, bottom-pinned last.
    let mut final_result = sticky_items;
    final_result.extend(result);
    final_result.extend(bottom_items);
    debug!("Final result has {} items", final_result.len());
    final_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_case_query_matches_all_caps_candidate() {
        // Smart-case regression guard: "Nasa Hq" must match "NASA HQ".
        let items = vec![
            Item::new("NASA HQ").subtitle("Room"),
            Item::new("Unrelated").subtitle("Room"),
        ];
        let result = filter_and_sort_items(items, "Nasa Hq".to_string());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "NASA HQ");
    }

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
    fn test_filter_with_bottom_pinned_items() {
        let items = vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Status Row")
                .subtitle("Job running")
                .pin_to_bottom(true),
            Item::new("Banana").subtitle("Fruit"),
        ];

        // Bottom-pinned items appear last and are exempt from filtering
        // (the query doesn't match "Status Row" but it survives anyway).
        let result = filter_and_sort_items(items, "fruit".to_string());
        assert_eq!(result.len(), 3);
        assert_eq!(result[2].title, "Status Row");
        assert!(result[..2].iter().any(|item| item.title == "Apple"));
        assert!(result[..2].iter().any(|item| item.title == "Banana"));
    }

    #[test]
    fn test_sticky_regular_and_bottom_ordering() {
        let items = vec![
            Item::new("Footer").subtitle("Item").pin_to_bottom(true),
            Item::new("Normal").subtitle("Item"),
            Item::new("Header").subtitle("Item").sticky(true),
        ];

        let result = filter_and_sort_items(items, "item".to_string());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Header"); // sticky first
        assert_eq!(result[1].title, "Normal"); // filtered middle
        assert_eq!(result[2].title, "Footer"); // bottom-pinned last
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

    #[test]
    fn test_boost_affects_sort_order() {
        // Items with identical content but different boosts
        let items = vec![
            Item::new("Apple").subtitle("Fruit").boost(0),
            Item::new("Apple").subtitle("Fruit").boost(100),
        ];

        let result = filter_and_sort_items(items, "apple".to_string());
        assert_eq!(result.len(), 2);
        // Item with higher boost should be first
        assert_eq!(result[0].boost, 100);
        assert_eq!(result[1].boost, 0);
    }

    #[test]
    fn test_boost_can_overcome_fuzzy_score() {
        // "Apple" matches "apple" better than "Pineapple" does
        // But with enough boost, the worse match can rank higher
        let items = vec![
            Item::new("Apple").subtitle("Fruit").boost(0),
            Item::new("Pineapple").subtitle("Fruit").boost(200),
        ];

        let result = filter_and_sort_items(items, "apple".to_string());
        assert_eq!(result.len(), 2);
        // Despite worse fuzzy match, boosted item should be first
        assert_eq!(result[0].title, "Pineapple");
        assert_eq!(result[1].title, "Apple");
    }

    #[test]
    fn test_negative_boost_lowers_ranking() {
        let items = vec![
            Item::new("Apple").subtitle("Fruit").boost(-100),
            Item::new("Apple").subtitle("Fruit").boost(0),
        ];

        let result = filter_and_sort_items(items, "apple".to_string());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].boost, 0);
        assert_eq!(result[1].boost, -100);
    }

    #[test]
    fn test_boost_does_not_affect_sticky_items() {
        // Sticky items always appear first, regardless of boost
        let items = vec![
            Item::new("Normal").subtitle("Item").boost(1000),
            Item::new("Sticky").subtitle("Item").sticky(true).boost(0),
        ];

        let result = filter_and_sort_items(items, "item".to_string());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Sticky");
        assert_eq!(result[1].title, "Normal");
    }

    #[test]
    fn test_boost_only_affects_matching_items() {
        // Boost doesn't make non-matching items appear
        let items = vec![
            Item::new("Banana").subtitle("Fruit").boost(1000),
            Item::new("Apple").subtitle("Fruit").boost(0),
        ];

        let result = filter_and_sort_items(items, "apple".to_string());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Apple");
    }
}
