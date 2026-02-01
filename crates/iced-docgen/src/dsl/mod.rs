//! DSL module for declarative visual testing
//!
//! This module provides a high-level DSL for writing visual tests that generate
//! human-readable documentation alongside their execution.
//!
//! # Overview
//!
//! The DSL wraps `iced_test::Simulator` to provide:
//! - **Documented actions** with descriptions and expected outcomes
//! - **Widget introspection** for understanding UI state
//! - **Rich failure diagnostics** with available actions and suggestions
//! - **User story generation** from test execution traces
//! - **Vision AI integration** (optional) for AI-powered visual assertions
//!
//! # Example
//!
//! ```ignore
//! use iced_docgen::dsl::{TestAction, TestContext};
//!
//! let mut ctx = TestContext::new(my_view());
//!
//! ctx.execute(TestAction::click("Submit").described_as("Submit the form"))?;
//! ctx.execute(TestAction::expect("Success"))?;
//!
//! let story = ctx.to_story();
//! println!("{}", story.to_org());
//! ```
//!
//! # Vision AI (Optional)
//!
//! When the `vision` feature is enabled, you can use AI-powered visual assertions:
//!
//! ```ignore
//! use iced_docgen::dsl::vision::vision_client;
//!
//! if let Some(client) = vision_client() {
//!     let result = client.assert(&screenshot_path, "Is there a Submit button?");
//!     assert!(result.passed);
//! }
//! ```

mod action;
mod context;
mod diagnostics;
mod inspector;
mod story;

#[cfg(feature = "vision")]
pub mod vision;

pub use action::{ActionKind, CapturePoint, TestAction};
pub use context::{ActionTrace, ScenarioMeta, TestContext, UiState};
pub use diagnostics::{AvailableAction, FailureReason, TestFailure, TestHint};
pub use inspector::{WidgetKind, WidgetNode, WidgetTree};
pub use story::{StoryStep, UserStory};

// Re-export vision types at module level for convenience
#[cfg(feature = "vision")]
pub use vision::{
    AssertionResponse, UiDescription, VisionClient, VisionError, VisualAssertion, vision_client,
};
