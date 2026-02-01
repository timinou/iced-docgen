//! Registry for collecting documented items at compile time
//!
//! Uses the `inventory` crate to gather all items annotated with
//! iced-docgen macros across the entire crate graph.

use inventory;

/// A documented item entry collected at compile time
#[derive(Debug, Clone)]
pub struct DocEntry {
    /// What kind of item this is
    pub kind: DocKind,
    /// Unique identifier (usually the function/type name)
    pub id: &'static str,
    /// Human-readable title for documentation
    pub title: &'static str,
    /// Section this belongs to (views, models, usecases, etc.)
    pub section: &'static str,
    /// Description text (from doc comments or explicit)
    pub description: &'static str,
    /// Source file path
    pub source_file: &'static str,
    /// Source line number
    pub source_line: u32,
    /// Tags for categorization and search
    pub tags: &'static [&'static str],
    /// Types/items this links to
    pub links_to: &'static [&'static str],
    /// Related items to reference
    pub see_also: &'static [&'static str],
    /// Type-specific metadata
    pub metadata: DocMetadata,
}

inventory::collect!(DocEntry);

/// Get all registered documentation entries
pub fn all_entries() -> impl Iterator<Item = &'static DocEntry> {
    inventory::iter::<DocEntry>.into_iter()
}

/// Get entries filtered by section
pub fn entries_by_section(section: &str) -> Vec<&'static DocEntry> {
    all_entries().filter(|e| e.section == section).collect()
}

/// Get entries filtered by kind
pub fn entries_by_kind(kind: DocKind) -> Vec<&'static DocEntry> {
    all_entries().filter(|e| e.kind == kind).collect()
}

/// Get entries filtered by tag
pub fn entries_by_tag(tag: &str) -> Vec<&'static DocEntry> {
    all_entries().filter(|e| e.tags.contains(&tag)).collect()
}

/// The kind of documented item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    /// A function (view, helper, etc.)
    Function,
    /// A struct
    Struct,
    /// An enum
    Enum,
    /// A module
    Module,
    /// A user story / use case
    Usecase,
    /// A multi-step workflow
    Workflow,
    /// An .ice end-to-end test file
    IceTest,
    /// A test scenario (via #[scenario] macro)
    Scenario,
    /// A user story test (via #[user_story] macro)
    UserStory,
}

/// Type-specific metadata for documented items
#[derive(Debug, Clone)]
pub enum DocMetadata {
    /// No additional metadata
    None,
    /// Screenshot metadata
    Screenshot(ScreenshotMeta),
    /// Use case metadata
    Usecase(UsecaseMeta),
    /// Workflow metadata
    Workflow(WorkflowMeta),
    /// State machine metadata
    State(StateMeta),
    /// Ice test file metadata
    IceTest(IceTestMeta),
    /// Test scenario metadata
    Scenario(ScenarioMeta),
    /// User story metadata
    UserStoryMeta(UserStoryMeta),
}

/// Metadata for screenshot-annotated views
#[derive(Debug, Clone)]
pub struct ScreenshotMeta {
    /// Base name for screenshot files
    pub name: &'static str,
    /// Theme to use (Light, Dark)
    pub theme: &'static str,
    /// Names of each scenario
    pub scenario_names: &'static [&'static str],
    /// State expressions for each scenario
    pub scenario_states: &'static [&'static str],
    /// Caption text for the screenshot
    pub caption: &'static str,
}

/// Metadata for use case documentation
#[derive(Debug, Clone)]
pub struct UsecaseMeta {
    /// Who performs this action
    pub actor: &'static str,
    /// What they want to achieve
    pub goal: &'static str,
    /// What must be true before
    pub preconditions: &'static [&'static str],
    /// Steps to perform
    pub steps: &'static [&'static str],
    /// What must be true after
    pub postconditions: &'static [&'static str],
}

/// Metadata for workflow documentation
#[derive(Debug, Clone)]
pub struct WorkflowMeta {
    /// User persona this workflow is for
    pub persona: &'static str,
    /// View names for each step
    pub step_views: &'static [&'static str],
    /// Action descriptions for each step
    pub step_actions: &'static [&'static str],
    /// Screenshot names for each step
    pub step_screenshots: &'static [&'static str],
    /// Step descriptions
    pub step_descriptions: &'static [&'static str],
    /// Expected outcomes
    pub outcomes: &'static [&'static str],
}

/// Metadata for state machine documentation
#[derive(Debug, Clone)]
pub struct StateMeta {
    /// Initial state name
    pub initial: &'static str,
    /// Terminal state names
    pub terminal: &'static [&'static str],
    /// Names of all states
    pub state_names: &'static [&'static str],
    /// Descriptions of each state
    pub state_descriptions: &'static [&'static str],
    /// Colors for each state (hex)
    pub state_colors: &'static [&'static str],
    /// Comma-separated transitions for each state
    pub state_transitions: &'static [&'static str],
}

/// Metadata for .ice end-to-end test files
#[derive(Debug, Clone)]
pub struct IceTestMeta {
    /// Path to the .ice file
    pub file_path: String,
    /// Viewport dimensions (width, height)
    pub viewport: Option<(u32, u32)>,
    /// Test mode (Immediate, Patient)
    pub mode: String,
    /// Preset name for initial state
    pub preset: Option<String>,
    /// List of test instructions
    pub instructions: Vec<IceInstruction>,
}

/// A single instruction in an .ice test file
#[derive(Debug, Clone)]
pub struct IceInstruction {
    /// Instruction type: click, type, expect, tap, screenshot, wait
    pub kind: String,
    /// Target selector or text
    pub target: String,
    /// Optional value (for type, screenshot, wait)
    pub value: Option<String>,
}

/// Metadata for test scenarios (via #[scenario] macro)
///
/// Scenarios are individual test cases with rich metadata for documentation.
#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    /// Preconditions that must be true before the test
    pub preconditions: &'static [&'static str],
}

/// Metadata for user story tests (via #[user_story] macro)
///
/// User stories are multi-step test journeys with actor, goal, and outcomes.
#[derive(Debug, Clone)]
pub struct UserStoryMeta {
    /// Actor performing the story (e.g., "Developer", "Admin")
    pub actor: &'static str,
    /// What the actor wants to achieve
    pub goal: &'static str,
    /// Prerequisites for this story
    pub preconditions: &'static [&'static str],
    /// Expected outcomes after the story completes
    pub outcomes: &'static [&'static str],
}

/// A test scenario entry collected at compile time
///
/// This is the primary type for registering test scenarios via inventory.
#[derive(Debug, Clone)]
pub struct TestScenarioEntry {
    /// Unique identifier for the scenario
    pub id: &'static str,
    /// Human-readable title
    pub title: &'static str,
    /// Description of what the scenario tests
    pub description: &'static str,
    /// Actor performing the test (for user stories)
    pub actor: &'static str,
    /// Preconditions that must be true
    pub preconditions: &'static [&'static str],
    /// Expected outcomes
    pub outcomes: &'static [&'static str],
    /// Tags for categorization
    pub tags: &'static [&'static str],
    /// Source file path
    pub source_file: &'static str,
    /// Source line number
    pub source_line: u32,
}

inventory::collect!(TestScenarioEntry);

/// Get all registered test scenario entries
pub fn all_test_scenarios() -> impl Iterator<Item = &'static TestScenarioEntry> {
    inventory::iter::<TestScenarioEntry>.into_iter()
}

/// Get test scenarios filtered by tag
pub fn test_scenarios_by_tag(tag: &str) -> Vec<&'static TestScenarioEntry> {
    all_test_scenarios()
        .filter(|e| e.tags.contains(&tag))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_kind_equality() {
        assert_eq!(DocKind::Function, DocKind::Function);
        assert_ne!(DocKind::Function, DocKind::Struct);
    }

    #[test]
    fn test_entries_iteration() {
        // Should not panic even with no entries
        let _count = all_entries().count();
        // Just ensure iteration works without panic
    }
}
