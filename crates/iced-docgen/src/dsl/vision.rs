//! Optional Moondream vision AI integration with structured JSON output
//!
//! This module provides integration with the Moondream vision model via Ollama
//! for AI-powered visual assertions in tests.
//!
//! # Example
//!
//! ```ignore
//! use iced_docgen::dsl::vision::vision_client;
//!
//! if let Some(client) = vision_client() {
//!     let result = client.assert(screenshot_path, "Is there a Submit button?");
//!     if result.passed {
//!         println!("Found: {}", result.response.unwrap().description);
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

/// Default Ollama endpoint
const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Model name for Moondream
const MOONDREAM_MODEL: &str = "moondream";

/// Structured response from Moondream for assertions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResponse {
    /// Whether the queried element was found
    pub found: bool,
    /// Confidence level from 0.0 to 1.0
    pub confidence: f32,
    /// Human-readable description of what was found
    pub description: String,
}

/// Result of a visual assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAssertion {
    /// The question that was asked
    pub question: String,
    /// Response from the vision model (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<AssertionResponse>,
    /// Whether the assertion passed
    pub passed: bool,
    /// Error message if the assertion failed due to an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Structured UI description from vision analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDescription {
    /// 2-3 sentence summary of the UI
    pub summary: String,
    /// List of UI elements identified
    pub elements: Vec<String>,
}

/// Error type for vision operations
#[derive(Debug, Clone)]
pub enum VisionError {
    /// Ollama server is not available
    OllamaUnavailable(String),
    /// Moondream model is not installed
    ModelNotInstalled,
    /// Failed to read the image file
    ImageReadError(String),
    /// Failed to parse response from Moondream
    ParseError(String),
    /// HTTP request failed
    RequestError(String),
}

impl std::fmt::Display for VisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisionError::OllamaUnavailable(msg) => write!(f, "Ollama unavailable: {}", msg),
            VisionError::ModelNotInstalled => write!(f, "Moondream model not installed"),
            VisionError::ImageReadError(msg) => write!(f, "Failed to read image: {}", msg),
            VisionError::ParseError(msg) => write!(f, "Failed to parse response: {}", msg),
            VisionError::RequestError(msg) => write!(f, "HTTP request failed: {}", msg),
        }
    }
}

impl std::error::Error for VisionError {}

/// Ollama API request for vision models
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

/// Ollama API response
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Client for interacting with Moondream via Ollama
pub struct VisionClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl VisionClient {
    /// Create a new VisionClient with the default Ollama URL
    pub fn new() -> Result<Self, VisionError> {
        Self::with_url(DEFAULT_OLLAMA_URL)
    }

    /// Create a new VisionClient with a custom Ollama URL
    pub fn with_url(url: &str) -> Result<Self, VisionError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| VisionError::RequestError(e.to_string()))?;

        let vision_client = Self {
            base_url: url.to_string(),
            client,
        };

        // Check if Ollama is running
        vision_client.check_availability()?;

        Ok(vision_client)
    }

    /// Check if Ollama is available and Moondream is installed
    fn check_availability(&self) -> Result<(), VisionError> {
        let tags_url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&tags_url)
            .send()
            .map_err(|e| VisionError::OllamaUnavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VisionError::OllamaUnavailable(format!(
                "HTTP {}",
                response.status()
            )));
        }

        // Check if moondream model is available
        let body: serde_json::Value = response
            .json()
            .map_err(|e| VisionError::ParseError(e.to_string()))?;

        let models = body["models"]
            .as_array()
            .ok_or_else(|| VisionError::ParseError("No models array".to_string()))?;

        let has_moondream = models.iter().any(|m| {
            m["name"]
                .as_str()
                .map(|n| n.starts_with(MOONDREAM_MODEL))
                .unwrap_or(false)
        });

        if !has_moondream {
            return Err(VisionError::ModelNotInstalled);
        }

        Ok(())
    }

    /// Assert a visual condition with structured JSON response
    ///
    /// # Arguments
    /// * `image` - Path to the screenshot image
    /// * `question` - The assertion question (e.g., "Is there a Submit button?")
    ///
    /// # Returns
    /// A `VisualAssertion` containing the result
    pub fn assert(&self, image: &Path, question: &str) -> VisualAssertion {
        let prompt = format!(
            r#"Check: {}

Return ONLY a JSON object with this exact format (no other text):
{{"found": true, "confidence": 0.95, "description": "brief description"}}

Use true/false for found, 0.0-1.0 for confidence."#,
            question
        );

        match self.query_vision(image, &prompt) {
            Ok(response_text) => {
                match self.parse_assertion_response(&response_text) {
                    Ok(response) => {
                        let passed = response.found && response.confidence >= 0.5;
                        VisualAssertion {
                            question: question.to_string(),
                            response: Some(response),
                            passed,
                            error: None,
                        }
                    }
                    Err(e) => {
                        // Try to infer from raw text if JSON parsing fails
                        let lower = response_text.to_lowercase();
                        let found = lower.contains("yes")
                            || lower.contains("found")
                            || lower.contains("visible");
                        VisualAssertion {
                            question: question.to_string(),
                            response: Some(AssertionResponse {
                                found,
                                confidence: 0.5,
                                description: response_text.trim().to_string(),
                            }),
                            passed: found,
                            error: Some(format!("JSON parse fallback: {}", e)),
                        }
                    }
                }
            }
            Err(e) => VisualAssertion {
                question: question.to_string(),
                response: None,
                passed: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// Generate a structured UI description
    ///
    /// # Arguments
    /// * `image` - Path to the screenshot image
    ///
    /// # Returns
    /// A `UiDescription` with summary and identified elements
    pub fn describe_ui(&self, image: &Path) -> Result<UiDescription, VisionError> {
        let prompt = r#"Describe this UI for a user-friendly test report.

Return ONLY a JSON object with this exact format (no other text):
{"summary": "2-3 sentence description of what the user sees and can do", "elements": ["element1", "element2", "element3"]}

Focus on buttons, inputs, text, and interactive elements."#;

        let response_text = self.query_vision(image, prompt)?;
        self.parse_ui_description(&response_text)
    }

    /// Send a query to Moondream via Ollama
    fn query_vision(&self, image: &Path, prompt: &str) -> Result<String, VisionError> {
        // Read and encode image
        let image_data = std::fs::read(image)
            .map_err(|e| VisionError::ImageReadError(format!("{}: {}", image.display(), e)))?;
        let image_base64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);

        let request = OllamaRequest {
            model: MOONDREAM_MODEL.to_string(),
            prompt: prompt.to_string(),
            images: vec![image_base64],
            stream: false,
        };

        let generate_url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&generate_url)
            .json(&request)
            .send()
            .map_err(|e| VisionError::RequestError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VisionError::RequestError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .map_err(|e| VisionError::ParseError(e.to_string()))?;

        Ok(ollama_response.response)
    }

    /// Parse assertion response JSON
    fn parse_assertion_response(&self, text: &str) -> Result<AssertionResponse, VisionError> {
        // Try to find JSON in the response (model might add extra text)
        let json_str = extract_json(text).ok_or_else(|| {
            VisionError::ParseError(format!("No JSON found in response: {}", text))
        })?;

        serde_json::from_str(&json_str)
            .map_err(|e| VisionError::ParseError(format!("Invalid JSON: {} - {}", e, json_str)))
    }

    /// Parse UI description response JSON
    fn parse_ui_description(&self, text: &str) -> Result<UiDescription, VisionError> {
        let json_str = extract_json(text).ok_or_else(|| {
            VisionError::ParseError(format!("No JSON found in response: {}", text))
        })?;

        serde_json::from_str(&json_str)
            .map_err(|e| VisionError::ParseError(format!("Invalid JSON: {} - {}", e, json_str)))
    }
}

impl Default for VisionClient {
    fn default() -> Self {
        Self::new().expect("Failed to create VisionClient")
    }
}

/// Extract JSON object from text that may contain other content
fn extract_json(text: &str) -> Option<String> {
    // Find the first { and last matching }
    let start = text.find('{')?;
    let mut depth = 0;
    let mut end = start;

    for (i, c) in text[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if depth == 0 && end > start {
        Some(text[start..end].to_string())
    } else {
        None
    }
}

/// Cached vision client singleton
static VISION_CLIENT: OnceLock<Option<VisionClient>> = OnceLock::new();
static INIT_WARNING_SHOWN: OnceLock<bool> = OnceLock::new();

/// Get the cached vision client, or None if unavailable
///
/// This function returns a reference to a lazily-initialized VisionClient.
/// If Ollama is not running or Moondream is not installed, it returns None
/// and logs a warning once.
pub fn vision_client() -> Option<&'static VisionClient> {
    VISION_CLIENT
        .get_or_init(|| {
            match VisionClient::new() {
                Ok(client) => Some(client),
                Err(e) => {
                    // Show warning once
                    if INIT_WARNING_SHOWN.get().is_none() {
                        INIT_WARNING_SHOWN.get_or_init(|| {
                            eprintln!("\n{}", format_unavailable_warning(&e));
                            true
                        });
                    }
                    None
                }
            }
        })
        .as_ref()
}

/// Format a helpful warning message when vision AI is unavailable
fn format_unavailable_warning(error: &VisionError) -> String {
    let mut msg = String::from("╭────────────────────────────────────────────────────────────╮\n");
    msg.push_str("│  ⚠️  Vision AI Unavailable - Visual assertions skipped     │\n");
    msg.push_str("╰────────────────────────────────────────────────────────────╯\n\n");

    match error {
        VisionError::OllamaUnavailable(_) => {
            msg.push_str("Ollama is not running. To enable AI-powered visual testing:\n\n");
            msg.push_str(&installation_instructions());
        }
        VisionError::ModelNotInstalled => {
            msg.push_str("Ollama is running but Moondream model is not installed.\n\n");
            msg.push_str("Install Moondream:\n");
            msg.push_str("  ollama pull moondream\n\n");
            msg.push_str("Then run it:\n");
            msg.push_str("  ollama run moondream\n");
        }
        _ => {
            msg.push_str(&format!("Error: {}\n\n", error));
            msg.push_str(&installation_instructions());
        }
    }

    msg.push_str("\nTests will continue without visual assertions.\n");
    msg
}

/// Get installation instructions for Ollama and Moondream
pub fn installation_instructions() -> String {
    let mut instructions = String::new();

    instructions.push_str("┌─────────────────────────────────────────────────────────────┐\n");
    instructions.push_str("│  Moondream Installation Guide                               │\n");
    instructions.push_str("└─────────────────────────────────────────────────────────────┘\n\n");

    instructions.push_str("1. Install Ollama:\n\n");

    instructions.push_str("   Linux:\n");
    instructions.push_str("   curl -fsSL https://ollama.com/install.sh | sh\n\n");

    instructions.push_str("   macOS:\n");
    instructions.push_str("   brew install ollama\n");
    instructions.push_str("   # Or download from: https://ollama.com/download\n\n");

    instructions.push_str("2. Start Ollama server:\n");
    instructions.push_str("   ollama serve\n\n");

    instructions.push_str("3. Pull and run Moondream model:\n");
    instructions.push_str("   ollama pull moondream\n");
    instructions.push_str("   ollama run moondream\n\n");

    instructions.push_str("4. Verify installation:\n");
    instructions.push_str("   curl http://localhost:11434/api/tags\n\n");

    instructions.push_str("Note: Moondream is a small (~1.9GB) vision-language model\n");
    instructions.push_str("that runs efficiently on most modern hardware.\n");

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json() {
        let text = "Here is the result: {\"found\": true, \"confidence\": 0.9, \"description\": \"button visible\"} end";
        let json = extract_json(text).unwrap();
        assert_eq!(
            json,
            r#"{"found": true, "confidence": 0.9, "description": "button visible"}"#
        );
    }

    #[test]
    fn test_extract_json_nested() {
        let text = r#"{"outer": {"inner": "value"}}"#;
        let json = extract_json(text).unwrap();
        assert_eq!(json, text);
    }

    #[test]
    fn test_extract_json_no_json() {
        let text = "No JSON here";
        assert!(extract_json(text).is_none());
    }

    #[test]
    fn test_assertion_response_serde() {
        let response = AssertionResponse {
            found: true,
            confidence: 0.95,
            description: "Submit button in bottom right".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: AssertionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.found, true);
        assert!((parsed.confidence - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_visual_assertion_serde() {
        let assertion = VisualAssertion {
            question: "Is there a button?".to_string(),
            response: Some(AssertionResponse {
                found: true,
                confidence: 0.9,
                description: "Button found".to_string(),
            }),
            passed: true,
            error: None,
        };

        let json = serde_json::to_string_pretty(&assertion).unwrap();
        assert!(json.contains("Is there a button?"));
        assert!(!json.contains("error")); // skip_serializing_if works
    }

    #[test]
    fn test_ui_description_serde() {
        let desc = UiDescription {
            summary: "A dialog with form inputs".to_string(),
            elements: vec!["text input".to_string(), "submit button".to_string()],
        };

        let json = serde_json::to_string(&desc).unwrap();
        let parsed: UiDescription = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.elements.len(), 2);
    }

    #[test]
    fn test_installation_instructions() {
        let instructions = installation_instructions();
        assert!(instructions.contains("Linux"));
        assert!(instructions.contains("macOS"));
        assert!(instructions.contains("ollama pull moondream"));
    }
}
