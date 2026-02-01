//! Test context wrapping iced_test::Simulator
//!
//! `TestContext` provides a high-level interface for executing test actions
//! with automatic tracing, screenshot capture, and story generation.

use super::action::{ActionKind, TestAction};
use super::diagnostics::{AvailableAction, FailureReason, TestFailure};
use super::inspector::WidgetTree;
use super::story::{StoryStep, UserStory};

use iced_test::core::{Element, Theme};
use iced_test::renderer::Renderer;
use iced_test::{Error as IcedError, Simulator, core};

use std::path::PathBuf;

// Type aliases for convenience with default Theme and Renderer
type DefaultSimulator<'a, M> = Simulator<'a, M, core::Theme, Renderer>;
type DefaultElement<'a, M> = Element<'a, M, core::Theme, Renderer>;

/// Metadata for a test scenario
#[derive(Debug, Clone, Default)]
pub struct ScenarioMeta {
    /// Scenario title
    pub title: String,
    /// Description
    pub description: String,
    /// Actor performing the test
    pub actor: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Expected outcomes
    pub outcomes: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Source file
    pub source_file: Option<String>,
    /// Source line
    pub source_line: Option<u32>,
}

/// Captured UI state at a point in time
#[derive(Debug, Clone)]
pub struct UiState {
    /// Available clickable elements
    pub clickable: Vec<String>,
    /// Available text inputs
    pub text_inputs: Vec<String>,
    /// All visible text
    pub visible_text: Vec<String>,
}

/// Record of an executed action
#[derive(Debug, Clone)]
pub struct ActionTrace {
    /// The action that was executed
    pub action: TestAction,
    /// UI state before the action
    pub before: UiState,
    /// UI state after the action
    pub after: UiState,
    /// Whether the action succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Screenshot path if captured
    pub screenshot: Option<PathBuf>,
}

/// Test context wrapping Simulator with tracing and introspection
///
/// This uses the default Theme and Renderer types from iced_test.
pub struct TestContext<'a, M: Clone + 'static> {
    simulator: DefaultSimulator<'a, M>,
    scenario: ScenarioMeta,
    trace: Vec<ActionTrace>,
    screenshots: Vec<(String, PathBuf)>,
    screenshot_dir: PathBuf,
    screenshot_counter: usize,
    current_step: usize,
    current_step_desc: Option<String>,
}

impl<'a, M: Clone + 'static> TestContext<'a, M> {
    /// Create a new test context for the given element
    pub fn new(element: impl Into<DefaultElement<'a, M>>) -> Self {
        Self {
            simulator: DefaultSimulator::new(element),
            scenario: ScenarioMeta::default(),
            trace: Vec::new(),
            screenshots: Vec::new(),
            screenshot_dir: PathBuf::from("tests/visual/reports/screenshots"),
            screenshot_counter: 0,
            current_step: 0,
            current_step_desc: None,
        }
    }

    /// Set the screenshot output directory
    pub fn with_screenshot_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.screenshot_dir = dir.into();
        self
    }

    /// Set scenario metadata
    pub fn with_scenario(mut self, meta: ScenarioMeta) -> Self {
        self.scenario = meta;
        self
    }

    /// Begin a new step in the story
    ///
    /// Steps group related actions together for documentation purposes.
    pub fn step(&mut self, description: impl Into<String>) {
        self.current_step += 1;
        self.current_step_desc = Some(description.into());
    }

    /// Execute a test action with full tracing
    ///
    /// This method:
    /// 1. Captures UI state before the action
    /// 2. Optionally captures a screenshot before
    /// 3. Executes the action on the Simulator
    /// 4. Captures UI state after
    /// 5. Optionally captures a screenshot after
    /// 6. Records the trace for documentation
    ///
    /// Returns `Ok(())` if the action succeeded, or a `TestFailure` with rich diagnostics.
    pub fn execute(&mut self, action: TestAction) -> Result<(), TestFailure> {
        let before = self.capture_state();

        // Screenshot before if requested
        if action.capture.captures_before() {
            let _path = self.capture_screenshot(&format!("before_{}", self.trace.len()));
        }

        // Execute the action
        let result = self.run_action(&action);

        let after = self.capture_state();

        // Screenshot after if requested
        let screenshot = if action.capture.captures_after() {
            self.capture_screenshot(&format!("step_{}", self.trace.len()))
                .ok()
        } else {
            None
        };

        // Record the trace
        let trace = ActionTrace {
            action: action.clone(),
            before,
            after,
            success: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
            screenshot: screenshot.clone(),
        };
        self.trace.push(trace);

        // Handle failure
        if let Err(iced_error) = result {
            let failure = self.build_failure(action, iced_error, screenshot);
            return Err(failure);
        }

        Ok(())
    }

    /// Run the action on the underlying Simulator
    fn run_action(&mut self, action: &TestAction) -> Result<(), IcedError> {
        match &action.kind {
            ActionKind::Click(selector) => {
                self.simulator.click(selector.as_str())?;
            }
            ActionKind::Type(text) => {
                // typewrite returns event::Status, not Result
                let _ = self.simulator.typewrite(text);
            }
            ActionKind::Tap(key) => {
                // tap_key returns event::Status, not Result
                let _ = self.simulator.tap_key(*key);
            }
            ActionKind::Expect(text) => {
                self.simulator.find(text.as_str())?;
            }
            ActionKind::Wait(duration) => {
                std::thread::sleep(*duration);
            }
        }
        Ok(())
    }

    /// Build a TestFailure from an iced_test error
    fn build_failure(
        &mut self,
        action: TestAction,
        error: IcedError,
        screenshot: Option<PathBuf>,
    ) -> TestFailure {
        let reason = match &error {
            IcedError::SelectorNotFound { selector } => FailureReason::SelectorNotFound {
                selector: selector.clone(),
                similar: self.find_similar_selectors(selector),
            },
            _ => FailureReason::ExpectationFailed {
                expected: format!("{:?}", action.kind),
                found: None,
            },
        };

        let available = self.available_actions();
        let tree = self.inspect();

        let mut failure = TestFailure::new(action, reason)
            .with_available(available)
            .with_tree(tree);

        if let Some(path) = screenshot {
            failure = failure.with_screenshot(path);
        }

        failure.generate_hints();
        failure
    }

    /// Find selectors similar to the given one (for error suggestions)
    fn find_similar_selectors(&mut self, target: &str) -> Vec<String> {
        let state = self.capture_state();
        let target_lower = target.to_lowercase();

        let mut candidates: Vec<(String, usize)> = state
            .clickable
            .iter()
            .chain(state.visible_text.iter())
            .filter_map(|s| {
                let distance = levenshtein(&target_lower, &s.to_lowercase());
                if distance <= 5 {
                    Some((s.clone(), distance))
                } else if s.to_lowercase().contains(&target_lower)
                    || target_lower.contains(&s.to_lowercase())
                {
                    Some((s.clone(), 1))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|(_, d)| *d);
        candidates.into_iter().take(5).map(|(s, _)| s).collect()
    }

    /// Capture the current UI state
    fn capture_state(&mut self) -> UiState {
        // For now, return a basic state. In a full implementation,
        // we would use widget operations to enumerate elements.
        UiState {
            clickable: Vec::new(),
            text_inputs: Vec::new(),
            visible_text: Vec::new(),
        }
    }

    /// Get available actions in current UI state
    pub fn available_actions(&mut self) -> Vec<AvailableAction> {
        // In a full implementation, this would enumerate clickable elements
        // and text inputs using widget operations
        Vec::new()
    }

    /// Inspect the widget tree
    pub fn inspect(&mut self) -> WidgetTree {
        // In a full implementation, this would traverse the widget tree
        // using Simulator's find/operate methods
        WidgetTree::empty()
    }

    /// Capture a screenshot and return its path
    fn capture_screenshot(&mut self, name: &str) -> Result<PathBuf, TestFailure> {
        self.screenshot_counter += 1;
        let filename = format!(
            "{}_{}.png",
            self.scenario
                .title
                .replace(' ', "_")
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>(),
            name
        );

        let path = self.screenshot_dir.join(&filename);

        // Create the directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // Capture using Simulator's snapshot
        let _snapshot = self.simulator.snapshot(&Theme::Dark).map_err(|e| {
            TestFailure::new(
                TestAction::expect("screenshot"),
                FailureReason::Timeout {
                    condition: format!("screenshot capture: {:?}", e),
                },
            )
        })?;

        // Save to file
        // Note: In a real implementation, we would use snapshot.matches_image()
        // or write the bytes directly

        self.screenshots.push((name.to_string(), path.clone()));
        Ok(path)
    }

    /// Generate a user story from the execution trace
    pub fn to_story(&self) -> UserStory {
        let mut story = UserStory::new(&self.scenario.title)
            .with_actor(&self.scenario.actor)
            .with_goal(&self.scenario.description)
            .with_preconditions(self.scenario.preconditions.iter().cloned())
            .with_outcomes(self.scenario.outcomes.iter().cloned())
            .with_tags(self.scenario.tags.iter().cloned());

        if let (Some(file), Some(line)) = (&self.scenario.source_file, self.scenario.source_line) {
            story = story.with_source(file.clone(), line);
        }

        // Group traces into steps
        let mut current_step = 0;
        let mut step_actions: Vec<TestAction> = Vec::new();
        let mut step_screenshot: Option<PathBuf> = None;

        for trace in &self.trace {
            // Check if action has a description - if so, it's a new step
            if trace.action.description.is_some() {
                // Save previous step if any
                if !step_actions.is_empty() {
                    current_step += 1;
                    let step = StoryStep::new(current_step, "Untitled step")
                        .with_action(step_actions.remove(0));
                    story.add_step(step);
                    step_actions.clear();
                }
            }

            step_actions.push(trace.action.clone());
            if trace.screenshot.is_some() {
                step_screenshot = trace.screenshot.clone();
            }
        }

        // Add remaining actions as final step
        if !step_actions.is_empty() {
            current_step += 1;
            let desc = step_actions
                .first()
                .and_then(|a| a.description.clone())
                .unwrap_or_else(|| "Final step".to_string());

            let mut step = StoryStep::new(current_step, desc);
            for action in step_actions {
                step = step.with_action(action);
            }
            if let Some(path) = step_screenshot {
                step = step.with_screenshot(path);
            }
            story.add_step(step);
        }

        story
    }

    /// Get the execution trace
    pub fn trace(&self) -> &[ActionTrace] {
        &self.trace
    }

    /// Get captured screenshots
    pub fn screenshots(&self) -> &[(String, PathBuf)] {
        &self.screenshots
    }

    /// Get underlying simulator messages
    pub fn into_messages(self) -> impl Iterator<Item = M> {
        self.simulator.into_messages()
    }
}

/// Simple Levenshtein distance
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut matrix = vec![vec![0usize; b.len() + 1]; a.len() + 1];

    for i in 0..=a.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b.len() {
        matrix[0][j] = j;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_meta() {
        let meta = ScenarioMeta {
            title: "Test Scenario".to_string(),
            actor: "Developer".to_string(),
            ..Default::default()
        };
        assert_eq!(meta.title, "Test Scenario");
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein("cat", "cat"), 0);
        assert_eq!(levenshtein("cat", "bat"), 1);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }
}
