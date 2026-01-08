use alfrusco::config::TestingProvider;
use alfrusco::{execute, Item, Runnable, Workflow, WorkflowError};
use tempfile::TempDir;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Test error")]
struct TestError;

impl WorkflowError for TestError {}

// Runnable that adds sticky items and regular items
struct StickyItemsWorkflow;

impl Runnable for StickyItemsWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        // Add a sticky item
        let sticky_item = Item::new("Important Notice")
            .subtitle("This should always appear first")
            .sticky(true);
        workflow.append_item(sticky_item);

        // Add regular items
        workflow.append_item(Item::new("Apple").subtitle("A fruit"));
        workflow.append_item(Item::new("Banana").subtitle("A fruit"));
        workflow.append_item(Item::new("Carrot").subtitle("A vegetable"));

        // Enable filtering
        workflow.set_filter_keyword("fruit".to_string());

        Ok(())
    }
}

#[test]
fn test_workflow_filtering_with_sticky_items() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, StickyItemsWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    // Sticky item should be in the output
    assert!(result.contains("Important Notice"));
    // Filtered items should be in the output
    assert!(result.contains("Apple"));
    assert!(result.contains("Banana"));
    // Non-matching items should NOT be in the output
    assert!(
        !result.contains("Carrot"),
        "Carrot should be filtered out: {result}"
    );
}

// Runnable that adds only regular items (no sticky)
struct NoStickyItemsWorkflow;

impl Runnable for NoStickyItemsWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        workflow.append_item(Item::new("Dog").subtitle("A pet"));
        workflow.append_item(Item::new("Cat").subtitle("A pet"));
        workflow.append_item(Item::new("Fish").subtitle("An aquatic pet"));

        workflow.set_filter_keyword("pet".to_string());

        Ok(())
    }
}

#[test]
fn test_workflow_filtering_without_sticky_items() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, NoStickyItemsWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    // All items match "pet"
    assert!(result.contains("Dog"));
    assert!(result.contains("Cat"));
    assert!(result.contains("Fish"));
}

// Runnable with empty filter query
struct EmptyQueryWorkflow;

impl Runnable for EmptyQueryWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        workflow.append_item(Item::new("Item 1").subtitle("First"));
        workflow.append_item(Item::new("Item 2").subtitle("Second"));

        workflow.set_filter_keyword(String::new());

        Ok(())
    }
}

#[test]
fn test_workflow_filtering_empty_query() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, EmptyQueryWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    // Both items should be present (empty query matches all)
    assert!(result.contains("Item 1"));
    assert!(result.contains("Item 2"));
}

// Runnable with no filtering enabled
struct NoFilteringWorkflow;

impl Runnable for NoFilteringWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        workflow.append_item(Item::new("Apple").subtitle("Fruit"));
        workflow.append_item(Item::new("Carrot").subtitle("Vegetable"));
        // Don't set a filter keyword
        Ok(())
    }
}

#[test]
fn test_workflow_no_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, NoFilteringWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    assert!(result.contains("Apple"));
    assert!(result.contains("Carrot"));
}

// Runnable that checks sticky items come first
struct StickyFirstWorkflow;

impl Runnable for StickyFirstWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        // Add regular items first
        workflow.append_item(Item::new("Zebra").subtitle("Animal"));
        workflow.append_item(Item::new("Aardvark").subtitle("Animal"));

        // Add sticky item last
        let sticky = Item::new("PRIORITY")
            .subtitle("Should be first")
            .sticky(true);
        workflow.append_item(sticky);

        workflow.set_filter_keyword("animal".to_string());

        Ok(())
    }
}

#[test]
fn test_workflow_sticky_items_always_first() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, StickyFirstWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    // PRIORITY should appear before the other items in the JSON
    let priority_pos = result.find("PRIORITY").unwrap();
    let zebra_pos = result.find("Zebra").unwrap();
    let aardvark_pos = result.find("Aardvark").unwrap();

    assert!(
        priority_pos < zebra_pos,
        "Sticky item should come before Zebra"
    );
    assert!(
        priority_pos < aardvark_pos,
        "Sticky item should come before Aardvark"
    );
}

// Multiple sticky items test
struct MultipleStickyWorkflow;

impl Runnable for MultipleStickyWorkflow {
    type Error = TestError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        workflow.append_item(Item::new("Regular 1").subtitle("Normal item"));
        workflow.append_item(Item::new("Sticky A").subtitle("First sticky").sticky(true));
        workflow.append_item(Item::new("Regular 2").subtitle("Normal item"));
        workflow.append_item(Item::new("Sticky B").subtitle("Second sticky").sticky(true));

        workflow.set_filter_keyword("item".to_string());

        Ok(())
    }
}

#[test]
fn test_workflow_multiple_sticky_items() {
    let temp_dir = TempDir::new().unwrap();
    let provider = TestingProvider(temp_dir.path().to_path_buf());

    let mut output = Vec::new();
    execute(&provider, MultipleStickyWorkflow, &mut output);

    let result = String::from_utf8(output).unwrap();

    // Both sticky items should be present
    assert!(result.contains("Sticky A"));
    assert!(result.contains("Sticky B"));

    // Both regular items should be filtered in (they match "item")
    assert!(result.contains("Regular 1"));
    assert!(result.contains("Regular 2"));

    // Sticky items should come before regular items
    let sticky_a_pos = result.find("Sticky A").unwrap();
    let sticky_b_pos = result.find("Sticky B").unwrap();
    let regular_1_pos = result.find("Regular 1").unwrap();
    let regular_2_pos = result.find("Regular 2").unwrap();

    assert!(sticky_a_pos < regular_1_pos);
    assert!(sticky_a_pos < regular_2_pos);
    assert!(sticky_b_pos < regular_1_pos);
    assert!(sticky_b_pos < regular_2_pos);
}
