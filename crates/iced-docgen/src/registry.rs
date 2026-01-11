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
    all_entries()
        .filter(|e| e.tags.contains(&tag))
        .collect()
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
