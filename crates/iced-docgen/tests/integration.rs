//! Integration tests for iced-docgen macros
//!
//! These tests verify that the macros work correctly and that documentation
//! can be generated from annotated code.

use iced::widget::{column, text};
use iced::Element;
use iced_docgen::{documented, screenshot, state_doc, usecase, workflow};
use iced_docgen::{all_entries, generate, DocKind, DocMetadata, GenerateOptions};
use std::path::PathBuf;

// Simple test types
#[derive(Debug, Clone, Default)]
pub struct TestState {
    pub counter: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum TestMessage {
    Increment,
    Decrement,
}

// Test function with #[documented]
/// This is a test view that displays a counter
///
/// It shows the current value and allows incrementing.
#[documented(
    title = "Counter View",
    section = "views",
    tags = ["ui", "counter"],
    links_to = ["TestState", "TestMessage"],
    see_also = ["another_view"]
)]
pub fn counter_view(_state: &TestState) -> Element<'static, TestMessage> {
    column![text("Counter View")].into()
}

// Test function with #[screenshot]
/// Another test view for screenshots
#[screenshot(
    name = "test_view",
    theme = "Light",
    scenarios = [
        ("default", "TestState::default()"),
        ("with_value", "TestState { counter: 42, name: String::new() }")
    ],
    caption = "A test view showing screenshot scenarios"
)]
pub fn screenshot_view(_state: &TestState) -> Element<'static, TestMessage> {
    column![text("Screenshot View")].into()
}

// Test state machine with #[state_doc]
#[state_doc(
    title = "Test Status",
    description = "Status for testing state documentation",
    initial = "Pending",
    terminal = ["Done", "Cancelled"]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    #[state(description = "Not started yet", transitions_to = ["InProgress"])]
    Pending,

    #[state(description = "Currently working", transitions_to = ["Review", "Pending"])]
    InProgress,

    #[state(description = "Under review", transitions_to = ["Done", "InProgress"])]
    Review,

    #[state(description = "Completed successfully", terminal = true)]
    Done,

    #[state(description = "Cancelled by user", terminal = true)]
    Cancelled,
}

// Test usecase
#[usecase(
    title = "Increment Counter",
    actor = "User",
    goal = "Increase the counter value by one",
    preconditions = ["App is running", "Counter is visible"],
    steps = ["Click increment button", "Counter increases by 1"],
    postconditions = ["Counter value is increased"],
    tags = ["counter", "interaction"]
)]
#[test]
fn test_increment_counter() {
    let mut state = TestState::default();
    state.counter += 1;
    assert_eq!(state.counter, 1);
}

// Test workflow
#[allow(dead_code)]
#[workflow(
    title = "Basic Counter Workflow",
    description = "How users interact with the counter",
    persona = "End User",
    steps = [
        ("Counter", "View initial state", "workflow_1", "See the counter at 0"),
        ("Counter", "Click increment", "workflow_2", "Increase the counter")
    ],
    outcomes = ["User understands counter functionality"]
)]
fn document_counter_workflow() {}

// Tests
#[test]
fn test_documented_macro_registers_entry() {
    let entries: Vec<_> = all_entries().collect();

    // Find the counter_view entry
    let counter_entry = entries.iter().find(|e| e.id == "counter_view");
    assert!(
        counter_entry.is_some(),
        "counter_view should be registered. Found entries: {:?}",
        entries.iter().map(|e| e.id).collect::<Vec<_>>()
    );

    let entry = counter_entry.unwrap();
    assert_eq!(entry.title, "Counter View");
    assert_eq!(entry.section, "views");
    assert_eq!(entry.kind, DocKind::Function);
    assert!(entry.tags.contains(&"ui"));
    assert!(entry.tags.contains(&"counter"));
    assert!(entry.links_to.contains(&"TestState"));
}

#[test]
fn test_screenshot_macro_registers_metadata() {
    let entries: Vec<_> = all_entries().collect();

    let screenshot_entry = entries.iter().find(|e| e.id == "screenshot_view");
    assert!(
        screenshot_entry.is_some(),
        "screenshot_view should be registered"
    );

    let entry = screenshot_entry.unwrap();
    if let DocMetadata::Screenshot(meta) = &entry.metadata {
        assert_eq!(meta.name, "test_view");
        assert_eq!(meta.theme, "Light");
        assert_eq!(meta.scenario_names.len(), 2);
        assert!(meta.scenario_names.contains(&"default"));
        assert!(meta.scenario_names.contains(&"with_value"));
    } else {
        panic!("Expected Screenshot metadata");
    }
}

#[test]
fn test_state_doc_macro_registers_state_machine() {
    let entries: Vec<_> = all_entries().collect();

    let state_entry = entries.iter().find(|e| e.id == "TestStatus");
    assert!(state_entry.is_some(), "TestStatus should be registered");

    let entry = state_entry.unwrap();
    assert_eq!(entry.kind, DocKind::Enum);

    if let DocMetadata::State(meta) = &entry.metadata {
        assert_eq!(meta.initial, "Pending");
        assert!(meta.terminal.contains(&"Done"));
        assert!(meta.terminal.contains(&"Cancelled"));
        assert_eq!(meta.state_names.len(), 5);
    } else {
        panic!("Expected State metadata");
    }
}

#[test]
fn test_usecase_macro_registers_usecase() {
    let entries: Vec<_> = all_entries().collect();

    let usecase_entry = entries.iter().find(|e| e.id == "test_increment_counter");
    assert!(
        usecase_entry.is_some(),
        "test_increment_counter should be registered"
    );

    let entry = usecase_entry.unwrap();
    assert_eq!(entry.kind, DocKind::Usecase);

    if let DocMetadata::Usecase(meta) = &entry.metadata {
        assert_eq!(meta.actor, "User");
        assert!(meta.steps.len() >= 2);
    } else {
        panic!("Expected Usecase metadata");
    }
}

#[test]
fn test_workflow_macro_registers_workflow() {
    let entries: Vec<_> = all_entries().collect();

    let workflow_entry = entries.iter().find(|e| e.id == "document_counter_workflow");
    assert!(
        workflow_entry.is_some(),
        "document_counter_workflow should be registered"
    );

    let entry = workflow_entry.unwrap();
    assert_eq!(entry.kind, DocKind::Workflow);

    if let DocMetadata::Workflow(meta) = &entry.metadata {
        assert_eq!(meta.persona, "End User");
        assert_eq!(meta.step_views.len(), 2);
    } else {
        panic!("Expected Workflow metadata");
    }
}

#[test]
fn test_generate_creates_org_files() {
    let temp_dir = std::env::temp_dir().join("iced-docgen-test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let options = GenerateOptions {
        output_dir: temp_dir.clone(),
        screenshots_dir: PathBuf::from("screenshots"),
        project_name: "Test Project".to_string(),
        include_ai_export: false,
        ice_tests_dir: None,
    };

    let result = generate(options).unwrap();

    assert!(result.entries_processed > 0);
    assert!(!result.files_written.is_empty());

    // Check that index.org was created
    let index_path = temp_dir.join("index.org");
    assert!(index_path.exists(), "index.org should be created");

    let index_content = std::fs::read_to_string(&index_path).unwrap();
    assert!(index_content.contains("#+TITLE: Test Project Documentation"));

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}
