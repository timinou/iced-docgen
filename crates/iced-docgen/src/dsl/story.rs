//! User story generation from test execution
//!
//! Transforms test execution traces into human-readable documentation.

use super::action::TestAction;
use std::fmt::Write;
use std::path::PathBuf;

/// A user story generated from test execution
#[derive(Debug, Clone)]
pub struct UserStory {
    /// Story title
    pub title: String,
    /// Actor performing the story (e.g., "Developer", "Admin")
    pub actor: String,
    /// What the actor wants to achieve
    pub goal: String,
    /// Description of the story
    pub description: Option<String>,
    /// Prerequisites for this story
    pub preconditions: Vec<String>,
    /// Steps in the story
    pub steps: Vec<StoryStep>,
    /// Expected outcomes
    pub outcomes: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Source file reference
    pub source_file: Option<String>,
    /// Source line number
    pub source_line: Option<u32>,
}

/// A single step in a user story
#[derive(Debug, Clone)]
pub struct StoryStep {
    /// Step number (1-indexed)
    pub number: usize,
    /// Short description of the step
    pub description: String,
    /// Detailed explanation of what happens
    pub details: Option<String>,
    /// Actions performed in this step
    pub actions: Vec<TestAction>,
    /// Screenshot captured for this step
    pub screenshot: Option<PathBuf>,
    /// Whether this step passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl UserStory {
    /// Create a new user story
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            actor: "User".to_string(),
            goal: String::new(),
            description: None,
            preconditions: Vec::new(),
            steps: Vec::new(),
            outcomes: Vec::new(),
            tags: Vec::new(),
            source_file: None,
            source_line: None,
        }
    }

    /// Set the actor
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = actor.into();
        self
    }

    /// Set the goal
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = goal.into();
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a precondition
    pub fn with_precondition(mut self, pre: impl Into<String>) -> Self {
        self.preconditions.push(pre.into());
        self
    }

    /// Add preconditions
    pub fn with_preconditions(mut self, pres: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.preconditions.extend(pres.into_iter().map(Into::into));
        self
    }

    /// Add an outcome
    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.outcomes.push(outcome.into());
        self
    }

    /// Add outcomes
    pub fn with_outcomes(mut self, outcomes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.outcomes.extend(outcomes.into_iter().map(Into::into));
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Set source location
    pub fn with_source(mut self, file: impl Into<String>, line: u32) -> Self {
        self.source_file = Some(file.into());
        self.source_line = Some(line);
        self
    }

    /// Add a step to the story
    pub fn add_step(&mut self, step: StoryStep) {
        self.steps.push(step);
    }

    /// Check if all steps passed
    pub fn passed(&self) -> bool {
        self.steps.iter().all(|s| s.passed)
    }

    /// Get the first failed step, if any
    pub fn first_failure(&self) -> Option<&StoryStep> {
        self.steps.iter().find(|s| !s.passed)
    }

    /// Render to org-mode format
    pub fn to_org(&self) -> String {
        let mut output = String::new();

        // Tags string
        let tags_str = if self.tags.is_empty() {
            String::new()
        } else {
            format!(":{}:", self.tags.join(":"))
        };

        // Header with status
        let status = if self.passed() { "" } else { "FAILED " };
        writeln!(output, "* {}{} {}", status, self.title, tags_str).unwrap();

        // Properties drawer
        writeln!(output, ":PROPERTIES:").unwrap();
        writeln!(output, ":ACTOR: {}", self.actor).unwrap();
        if !self.goal.is_empty() {
            writeln!(output, ":GOAL: {}", self.goal).unwrap();
        }
        if let (Some(file), Some(line)) = (&self.source_file, self.source_line) {
            writeln!(
                output,
                ":SOURCE: [[file:{}::{}][{}:{}]]",
                file, line, file, line
            )
            .unwrap();
        }
        writeln!(output, ":END:").unwrap();
        writeln!(output).unwrap();

        // Description
        if let Some(ref desc) = self.description {
            writeln!(output, "{}", desc).unwrap();
            writeln!(output).unwrap();
        }

        // Preconditions
        if !self.preconditions.is_empty() {
            writeln!(output, "** Preconditions").unwrap();
            for pre in &self.preconditions {
                writeln!(output, "- {}", pre).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Steps
        writeln!(output, "** Steps").unwrap();
        writeln!(output).unwrap();
        for step in &self.steps {
            output.push_str(&step.to_org());
        }

        // Outcomes
        if !self.outcomes.is_empty() {
            writeln!(output, "** Outcomes").unwrap();
            for outcome in &self.outcomes {
                let checkbox = if self.passed() { "[X]" } else { "[ ]" };
                writeln!(output, "- {} {}", checkbox, outcome).unwrap();
            }
            writeln!(output).unwrap();
        }

        output
    }

    /// Render to markdown format
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        // Title
        writeln!(output, "# {}", self.title).unwrap();
        writeln!(output).unwrap();

        // Metadata
        writeln!(output, "**Actor:** {}", self.actor).unwrap();
        if !self.goal.is_empty() {
            writeln!(output, "**Goal:** {}", self.goal).unwrap();
        }
        if !self.tags.is_empty() {
            writeln!(output, "**Tags:** {}", self.tags.join(", ")).unwrap();
        }
        writeln!(output).unwrap();

        // Description
        if let Some(ref desc) = self.description {
            writeln!(output, "{}", desc).unwrap();
            writeln!(output).unwrap();
        }

        // Preconditions
        if !self.preconditions.is_empty() {
            writeln!(output, "## Preconditions").unwrap();
            for pre in &self.preconditions {
                writeln!(output, "- {}", pre).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Steps
        writeln!(output, "## Steps").unwrap();
        writeln!(output).unwrap();
        for step in &self.steps {
            output.push_str(&step.to_markdown());
        }

        // Outcomes
        if !self.outcomes.is_empty() {
            writeln!(output, "## Outcomes").unwrap();
            for outcome in &self.outcomes {
                let checkbox = if self.passed() { "[x]" } else { "[ ]" };
                writeln!(output, "- {} {}", checkbox, outcome).unwrap();
            }
            writeln!(output).unwrap();
        }

        output
    }
}

impl StoryStep {
    /// Create a new step
    pub fn new(number: usize, description: impl Into<String>) -> Self {
        Self {
            number,
            description: description.into(),
            details: None,
            actions: Vec::new(),
            screenshot: None,
            passed: true,
            error: None,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add an action
    pub fn with_action(mut self, action: TestAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add a screenshot
    pub fn with_screenshot(mut self, path: PathBuf) -> Self {
        self.screenshot = Some(path);
        self
    }

    /// Mark as failed
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.passed = false;
        self.error = Some(error.into());
        self
    }

    /// Render to org-mode
    pub fn to_org(&self) -> String {
        let mut output = String::new();

        // Step heading
        let status = if self.passed { "" } else { " :fail:" };
        writeln!(
            output,
            "*** Step {}: {}{}",
            self.number, self.description, status
        )
        .unwrap();

        // Details
        if let Some(ref details) = self.details {
            writeln!(output, "{}", details).unwrap();
            writeln!(output).unwrap();
        }

        // Screenshot
        if let Some(ref screenshot) = self.screenshot {
            writeln!(output, "#+CAPTION: Step {} screenshot", self.number).unwrap();
            writeln!(output, "[[file:{}]]", screenshot.display()).unwrap();
            writeln!(output).unwrap();
        }

        // Actions performed
        if !self.actions.is_empty() {
            writeln!(output, "#+begin_src").unwrap();
            for action in &self.actions {
                writeln!(output, "{}", action.display_short()).unwrap();
            }
            writeln!(output, "#+end_src").unwrap();
            writeln!(output).unwrap();
        }

        // Error if failed
        if let Some(ref error) = self.error {
            writeln!(output, "#+begin_quote").unwrap();
            writeln!(output, "ERROR: {}", error).unwrap();
            writeln!(output, "#+end_quote").unwrap();
            writeln!(output).unwrap();
        }

        output
    }

    /// Render to markdown
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        // Step heading
        let status = if self.passed { "" } else { " ❌" };
        writeln!(
            output,
            "### Step {}: {}{}",
            self.number, self.description, status
        )
        .unwrap();
        writeln!(output).unwrap();

        // Details
        if let Some(ref details) = self.details {
            writeln!(output, "{}", details).unwrap();
            writeln!(output).unwrap();
        }

        // Screenshot
        if let Some(ref screenshot) = self.screenshot {
            writeln!(
                output,
                "![Step {} screenshot]({})",
                self.number,
                screenshot.display()
            )
            .unwrap();
            writeln!(output).unwrap();
        }

        // Actions
        if !self.actions.is_empty() {
            writeln!(output, "```").unwrap();
            for action in &self.actions {
                writeln!(output, "{}", action.display_short()).unwrap();
            }
            writeln!(output, "```").unwrap();
            writeln!(output).unwrap();
        }

        // Error
        if let Some(ref error) = self.error {
            writeln!(output, "> **ERROR:** {}", error).unwrap();
            writeln!(output).unwrap();
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::action::TestAction;

    #[test]
    fn test_story_building() {
        let story = UserStory::new("Login Flow")
            .with_actor("Developer")
            .with_goal("Access the dashboard")
            .with_preconditions(["Account exists", "Not logged in"])
            .with_outcomes(["User sees dashboard", "Session created"]);

        assert_eq!(story.title, "Login Flow");
        assert_eq!(story.actor, "Developer");
        assert_eq!(story.preconditions.len(), 2);
        assert_eq!(story.outcomes.len(), 2);
    }

    #[test]
    fn test_story_org_output() {
        let mut story = UserStory::new("Test Story")
            .with_actor("Tester")
            .with_tags(["test", "example"]);

        story.add_step(
            StoryStep::new(1, "Do something")
                .with_action(TestAction::click("Button"))
                .with_details("Click the button to proceed"),
        );

        let org = story.to_org();
        assert!(org.contains("* Test Story"));
        assert!(org.contains(":ACTOR: Tester"));
        assert!(org.contains("*** Step 1: Do something"));
        assert!(org.contains("click \"Button\""));
    }
}
