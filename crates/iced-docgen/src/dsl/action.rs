//! Test action wrapper with documentation metadata
//!
//! `TestAction` wraps `iced_test::Instruction` to add:
//! - Human-readable descriptions
//! - Expected outcomes
//! - Screenshot capture points

use iced_test::instruction::Key;
use std::time::Duration;

/// A test action with documentation metadata
#[derive(Debug, Clone)]
pub struct TestAction {
    /// The kind of action to perform
    pub kind: ActionKind,
    /// Human-readable description of what this action does
    pub description: Option<String>,
    /// What we expect to happen after this action
    pub expected: Option<String>,
    /// When to capture screenshots relative to this action
    pub capture: CapturePoint,
}

/// The kind of test action
#[derive(Debug, Clone)]
pub enum ActionKind {
    /// Click on an element matching the selector
    Click(String),
    /// Type text into the focused element
    Type(String),
    /// Tap a special key
    Tap(Key),
    /// Expect text to be present in the UI
    Expect(String),
    /// Wait for a duration (for async operations)
    Wait(Duration),
}

/// When to capture screenshots relative to an action
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CapturePoint {
    /// Don't capture any screenshot
    #[default]
    None,
    /// Capture before the action
    Before,
    /// Capture after the action
    After,
    /// Capture both before and after
    Both,
}

impl CapturePoint {
    /// Returns true if we should capture before the action
    pub fn captures_before(&self) -> bool {
        matches!(self, Self::Before | Self::Both)
    }

    /// Returns true if we should capture after the action
    pub fn captures_after(&self) -> bool {
        matches!(self, Self::After | Self::Both)
    }
}

impl TestAction {
    /// Create a click action for the given selector
    ///
    /// The selector can be any text that appears in a clickable element.
    ///
    /// # Example
    /// ```ignore
    /// TestAction::click("Submit")
    /// TestAction::click("Cancel")
    /// ```
    pub fn click(selector: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Click(selector.into()),
            description: None,
            expected: None,
            capture: CapturePoint::None,
        }
    }

    /// Create a type action to enter text
    ///
    /// This types the given text into the currently focused text input.
    ///
    /// # Example
    /// ```ignore
    /// TestAction::typewrite("Hello, world!")
    /// ```
    pub fn typewrite(text: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Type(text.into()),
            description: None,
            expected: None,
            capture: CapturePoint::None,
        }
    }

    /// Create a key tap action
    ///
    /// # Example
    /// ```ignore
    /// TestAction::tap(Key::Enter)
    /// TestAction::tap(Key::Tab)
    /// ```
    pub fn tap(key: Key) -> Self {
        Self {
            kind: ActionKind::Tap(key),
            description: None,
            expected: None,
            capture: CapturePoint::None,
        }
    }

    /// Create an expectation that text should be present
    ///
    /// # Example
    /// ```ignore
    /// TestAction::expect("Success!")
    /// TestAction::expect("Welcome, Alice")
    /// ```
    pub fn expect(text: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Expect(text.into()),
            description: None,
            expected: None,
            capture: CapturePoint::None,
        }
    }

    /// Create a wait action for async operations
    ///
    /// # Example
    /// ```ignore
    /// TestAction::wait(Duration::from_millis(100))
    /// ```
    pub fn wait(duration: Duration) -> Self {
        Self {
            kind: ActionKind::Wait(duration),
            description: None,
            expected: None,
            capture: CapturePoint::None,
        }
    }

    /// Add a human-readable description
    ///
    /// This description appears in generated documentation.
    ///
    /// # Example
    /// ```ignore
    /// TestAction::click("Login")
    ///     .described_as("Submit login credentials")
    /// ```
    pub fn described_as(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add an expected outcome description
    ///
    /// # Example
    /// ```ignore
    /// TestAction::click("Save")
    ///     .expecting("File saved confirmation appears")
    /// ```
    pub fn expecting(mut self, exp: impl Into<String>) -> Self {
        self.expected = Some(exp.into());
        self
    }

    /// Capture a screenshot after this action
    ///
    /// # Example
    /// ```ignore
    /// TestAction::click("Open Dashboard")
    ///     .with_screenshot()
    /// ```
    pub fn with_screenshot(mut self) -> Self {
        self.capture = CapturePoint::After;
        self
    }

    /// Capture a screenshot before this action
    pub fn with_screenshot_before(mut self) -> Self {
        self.capture = CapturePoint::Before;
        self
    }

    /// Capture screenshots both before and after this action
    pub fn with_screenshot_both(mut self) -> Self {
        self.capture = CapturePoint::Both;
        self
    }

    /// Set the capture point explicitly
    pub fn with_capture(mut self, capture: CapturePoint) -> Self {
        self.capture = capture;
        self
    }

    /// Get a short display string for this action
    pub fn display_short(&self) -> String {
        match &self.kind {
            ActionKind::Click(sel) => format!("click \"{}\"", sel),
            ActionKind::Type(text) => format!("type \"{}\"", truncate(text, 20)),
            ActionKind::Tap(key) => format!("tap {:?}", key),
            ActionKind::Expect(text) => format!("expect \"{}\"", truncate(text, 20)),
            ActionKind::Wait(d) => format!("wait {}ms", d.as_millis()),
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

impl std::fmt::Display for TestAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_short())?;
        if let Some(ref desc) = self.description {
            write!(f, " ({})", desc)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_builders() {
        let action = TestAction::click("Button")
            .described_as("Click the button")
            .expecting("Something happens")
            .with_screenshot();

        assert!(matches!(action.kind, ActionKind::Click(ref s) if s == "Button"));
        assert_eq!(action.description.as_deref(), Some("Click the button"));
        assert_eq!(action.expected.as_deref(), Some("Something happens"));
        assert_eq!(action.capture, CapturePoint::After);
    }

    #[test]
    fn test_capture_point() {
        assert!(!CapturePoint::None.captures_before());
        assert!(!CapturePoint::None.captures_after());
        assert!(CapturePoint::Before.captures_before());
        assert!(!CapturePoint::Before.captures_after());
        assert!(!CapturePoint::After.captures_before());
        assert!(CapturePoint::After.captures_after());
        assert!(CapturePoint::Both.captures_before());
        assert!(CapturePoint::Both.captures_after());
    }
}
