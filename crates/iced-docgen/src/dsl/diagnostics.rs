//! Rich failure diagnostics for test failures
//!
//! When a test fails, this module provides detailed context including:
//! - Available actions in the current UI state
//! - Widget tree visualization
//! - Suggested fixes based on common patterns

use super::action::TestAction;
use super::inspector::WidgetTree;
use std::fmt::Write;
use std::path::PathBuf;

/// A test failure with rich diagnostic context
#[derive(Debug)]
pub struct TestFailure {
    /// The action that failed
    pub action: TestAction,
    /// The reason for the failure
    pub reason: FailureReason,
    /// Actions that could have been performed instead
    pub available: Vec<AvailableAction>,
    /// Current widget tree state
    pub widget_tree: WidgetTree,
    /// Screenshot captured at failure point, if any
    pub screenshot: Option<PathBuf>,
    /// Suggested fixes
    pub hints: Vec<TestHint>,
}

/// The reason a test action failed
#[derive(Debug, Clone)]
pub enum FailureReason {
    /// The selector didn't match any element
    SelectorNotFound {
        /// The selector that was used
        selector: String,
        /// Similar selectors that were found
        similar: Vec<String>,
    },
    /// The element was found but not visible
    NotVisible {
        /// The selector
        selector: String,
        /// The element's bounds (may be outside viewport)
        bounds: Option<(f32, f32, f32, f32)>,
    },
    /// An expectation was not met
    ExpectationFailed {
        /// What was expected
        expected: String,
        /// What was actually found (if anything)
        found: Option<String>,
    },
    /// A timeout occurred waiting for something
    Timeout {
        /// What we were waiting for
        condition: String,
    },
    /// The element is not interactive
    NotInteractive {
        /// The selector
        selector: String,
        /// What kind of element it is
        element_kind: String,
    },
}

/// An action available in the current UI state
#[derive(Debug, Clone)]
pub struct AvailableAction {
    /// Selector to use for this action
    pub selector: String,
    /// Description of what the element is
    pub description: String,
    /// The kind of action possible (click, type, etc.)
    pub action_kind: String,
}

/// A hint for fixing a test failure
#[derive(Debug, Clone)]
pub struct TestHint {
    /// The suggestion
    pub message: String,
    /// Suggested code snippet, if applicable
    pub code: Option<String>,
    /// Priority (lower = more relevant)
    pub priority: u8,
}

impl TestFailure {
    /// Create a new test failure
    pub fn new(action: TestAction, reason: FailureReason) -> Self {
        Self {
            action,
            reason,
            available: Vec::new(),
            widget_tree: WidgetTree::empty(),
            screenshot: None,
            hints: Vec::new(),
        }
    }

    /// Add available actions
    pub fn with_available(mut self, available: Vec<AvailableAction>) -> Self {
        self.available = available;
        self
    }

    /// Add widget tree
    pub fn with_tree(mut self, tree: WidgetTree) -> Self {
        self.widget_tree = tree;
        self
    }

    /// Add screenshot path
    pub fn with_screenshot(mut self, path: PathBuf) -> Self {
        self.screenshot = Some(path);
        self
    }

    /// Generate hints based on the failure context
    pub fn generate_hints(&mut self) {
        self.hints.clear();

        match &self.reason {
            FailureReason::SelectorNotFound { selector, similar } => {
                // Suggest similar selectors
                for (i, sim) in similar.iter().take(3).enumerate() {
                    self.hints.push(TestHint {
                        message: format!("Did you mean \"{}\"?", sim),
                        code: Some(format!("TestAction::click(\"{}\")", sim)),
                        priority: i as u8,
                    });
                }

                // Suggest checking available actions
                if !self.available.is_empty() {
                    self.hints.push(TestHint {
                        message: "Check the available actions table below".to_string(),
                        code: None,
                        priority: 10,
                    });
                }

                // If looking for text, might be a timing issue
                self.hints.push(TestHint {
                    message: format!(
                        "If \"{}\" should appear after an async operation, try adding a wait",
                        selector
                    ),
                    code: Some("TestAction::wait(Duration::from_millis(100))".to_string()),
                    priority: 20,
                });
            }
            FailureReason::ExpectationFailed { expected, found } => {
                if let Some(actual) = found {
                    // Show what was found
                    self.hints.push(TestHint {
                        message: format!("Found \"{}\" instead of \"{}\"", actual, expected),
                        code: None,
                        priority: 0,
                    });

                    // Suggest updating expectation if it looks like a typo
                    if levenshtein(expected, actual) <= 3 {
                        self.hints.push(TestHint {
                            message: "This looks like a typo - consider updating the expectation"
                                .to_string(),
                            code: Some(format!("TestAction::expect(\"{}\")", actual)),
                            priority: 1,
                        });
                    }
                } else {
                    // Nothing found
                    self.hints.push(TestHint {
                        message: "The expected text was not found anywhere in the UI".to_string(),
                        code: None,
                        priority: 0,
                    });

                    // List available text
                    let texts = self.widget_tree.all_text();
                    if !texts.is_empty() {
                        self.hints.push(TestHint {
                            message: format!(
                                "Available text: {}",
                                texts
                                    .iter()
                                    .take(5)
                                    .map(|s| format!("\"{}\"", s))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            code: None,
                            priority: 5,
                        });
                    }
                }
            }
            FailureReason::NotInteractive {
                selector,
                element_kind,
            } => {
                self.hints.push(TestHint {
                    message: format!(
                        "\"{}\" is a {} which cannot be clicked",
                        selector, element_kind
                    ),
                    code: None,
                    priority: 0,
                });

                // Suggest looking for a nearby button
                for action in &self.available {
                    if action.action_kind == "click" {
                        self.hints.push(TestHint {
                            message: format!("Try clicking \"{}\" instead", action.selector),
                            code: Some(format!("TestAction::click(\"{}\")", action.selector)),
                            priority: 5,
                        });
                        break;
                    }
                }
            }
            FailureReason::Timeout { condition } => {
                self.hints.push(TestHint {
                    message: format!("Timeout waiting for: {}", condition),
                    code: None,
                    priority: 0,
                });
                self.hints.push(TestHint {
                    message: "Consider increasing the wait duration or checking async operations"
                        .to_string(),
                    code: None,
                    priority: 5,
                });
            }
            FailureReason::NotVisible { selector, bounds } => {
                if let Some((x, y, w, h)) = bounds {
                    self.hints.push(TestHint {
                        message: format!(
                            "\"{}\" exists but is not visible (bounds: {:.0}x{:.0} at ({:.0}, {:.0}))",
                            selector, w, h, x, y
                        ),
                        code: None,
                        priority: 0,
                    });
                }
                self.hints.push(TestHint {
                    message: "The element may need to be scrolled into view".to_string(),
                    code: None,
                    priority: 5,
                });
            }
        }

        // Sort hints by priority
        self.hints.sort_by_key(|h| h.priority);
    }

    /// Render to org-mode format
    pub fn to_org(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "* FAILED: {} :fail:", self.action.display_short()).unwrap();
        writeln!(output, ":PROPERTIES:").unwrap();
        writeln!(output, ":END:").unwrap();
        writeln!(output).unwrap();

        // Error description
        writeln!(output, "** Error").unwrap();
        writeln!(output, "#+begin_quote").unwrap();
        writeln!(output, "{}", self.reason_description()).unwrap();
        writeln!(output, "#+end_quote").unwrap();
        writeln!(output).unwrap();

        // Available actions table
        if !self.available.is_empty() {
            writeln!(output, "** Available Actions").unwrap();
            writeln!(output, "| Selector | Description | Action |").unwrap();
            writeln!(output, "|----------+-------------+--------|").unwrap();
            for action in &self.available {
                writeln!(
                    output,
                    "| \"{}\" | {} | {} |",
                    action.selector, action.description, action.action_kind
                )
                .unwrap();
            }
            writeln!(output).unwrap();
        }

        // Widget tree
        writeln!(output, "** Widget Tree").unwrap();
        output.push_str(&self.widget_tree.to_org());
        writeln!(output).unwrap();

        // Screenshot
        if let Some(ref screenshot) = self.screenshot {
            writeln!(output, "** Screenshot").unwrap();
            writeln!(output, "[[file:{}]]", screenshot.display()).unwrap();
            writeln!(output).unwrap();
        }

        // Suggestions
        if !self.hints.is_empty() {
            writeln!(output, "** Suggestions").unwrap();
            for (i, hint) in self.hints.iter().enumerate() {
                write!(output, "{}. {}", i + 1, hint.message).unwrap();
                if let Some(ref code) = hint.code {
                    writeln!(output).unwrap();
                    writeln!(output, "   #+begin_src rust").unwrap();
                    writeln!(output, "   {}", code).unwrap();
                    writeln!(output, "   #+end_src").unwrap();
                } else {
                    writeln!(output).unwrap();
                }
            }
        }

        output
    }

    /// Render to terminal with ANSI colors
    pub fn to_terminal(&self) -> String {
        let mut output = String::new();

        // Error header
        writeln!(output, "\x1b[1;31mFAILED\x1b[0m: {}", self.action).unwrap();
        writeln!(output).unwrap();

        // Reason
        writeln!(
            output,
            "\x1b[1mReason:\x1b[0m {}",
            self.reason_description()
        )
        .unwrap();
        writeln!(output).unwrap();

        // Available actions (if any)
        if !self.available.is_empty() {
            writeln!(output, "\x1b[1mAvailable actions:\x1b[0m").unwrap();
            for action in &self.available {
                writeln!(
                    output,
                    "  \x1b[36m\"{}\"\x1b[0m - {} ({})",
                    action.selector, action.description, action.action_kind
                )
                .unwrap();
            }
            writeln!(output).unwrap();
        }

        // Hints
        if !self.hints.is_empty() {
            writeln!(output, "\x1b[1mSuggestions:\x1b[0m").unwrap();
            for hint in &self.hints {
                writeln!(output, "  • {}", hint.message).unwrap();
                if let Some(ref code) = hint.code {
                    writeln!(output, "    \x1b[33m{}\x1b[0m", code).unwrap();
                }
            }
        }

        output
    }

    fn reason_description(&self) -> String {
        match &self.reason {
            FailureReason::SelectorNotFound { selector, .. } => {
                format!("Could not find element matching \"{}\"", selector)
            }
            FailureReason::NotVisible { selector, .. } => {
                format!("Element \"{}\" exists but is not visible", selector)
            }
            FailureReason::ExpectationFailed { expected, found } => {
                if let Some(actual) = found {
                    format!("Expected \"{}\" but found \"{}\"", expected, actual)
                } else {
                    format!("Expected \"{}\" but it was not found", expected)
                }
            }
            FailureReason::Timeout { condition } => {
                format!("Timeout waiting for: {}", condition)
            }
            FailureReason::NotInteractive {
                selector,
                element_kind,
            } => {
                format!(
                    "Element \"{}\" ({}) is not interactive",
                    selector, element_kind
                )
            }
        }
    }
}

/// Simple Levenshtein distance for typo detection
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

impl std::fmt::Display for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_terminal())
    }
}

impl std::error::Error for TestFailure {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::action::TestAction;

    #[test]
    fn test_failure_hints_selector_not_found() {
        let mut failure = TestFailure::new(
            TestAction::click("Submit"),
            FailureReason::SelectorNotFound {
                selector: "Submit".to_string(),
                similar: vec!["Send".to_string(), "OK".to_string()],
            },
        );
        failure.generate_hints();

        assert!(!failure.hints.is_empty());
        assert!(failure.hints[0].message.contains("Did you mean"));
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("cat", "cat"), 0);
        assert_eq!(levenshtein("cat", "bat"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }
}
