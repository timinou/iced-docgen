//! iced-docgen: Generate org-mode documentation from annotated Iced applications
//!
//! This crate provides procedural macros and runtime support for generating
//! org-mode documentation directly from your Rust code.
//!
//! # Quick Start
//!
//! ```ignore
//! use iced_docgen::{documented, screenshot};
//!
//! #[documented(
//!     title = "My View",
//!     section = "views",
//!     tags = ["ui"]
//! )]
//! #[screenshot(name = "my_view", theme = "Light")]
//! pub fn my_view(state: &AppState) -> Element<Message> {
//!     // ...
//! }
//! ```
//!
//! # Generating Documentation
//!
//! ```ignore
//! // In tests/generate_docs.rs
//! #[test]
//! fn generate_docs() {
//!     iced_docgen::generate(GenerateOptions::default()).unwrap();
//! }
//! ```

mod registry;
mod render;

pub use inventory;
pub use registry::*;
pub use render::*;

// Re-export macros
pub use iced_docgen_macros::{documented, screenshot, state, state_doc, usecase, workflow};

use std::io;
use std::path::PathBuf;

/// Options for documentation generation
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Output directory for generated .org files
    pub output_dir: PathBuf,
    /// Directory containing screenshots (for linking)
    pub screenshots_dir: PathBuf,
    /// Project name (used in org-mode title)
    pub project_name: String,
    /// Whether to generate AI export files
    pub include_ai_export: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs"),
            screenshots_dir: PathBuf::from("screenshots"),
            project_name: "Project".to_string(),
            include_ai_export: false,
        }
    }
}

/// Generate documentation from all registered entries
pub fn generate(options: GenerateOptions) -> io::Result<GenerateResult> {
    let entries: Vec<&DocEntry> = all_entries().collect();
    let renderer = OrgRenderer::new(&options);

    // Group entries by section
    let mut sections: std::collections::HashMap<&str, Vec<&DocEntry>> =
        std::collections::HashMap::new();

    for entry in &entries {
        sections.entry(entry.section).or_default().push(entry);
    }

    let mut files_written = vec![];

    // Create output directory
    std::fs::create_dir_all(&options.output_dir)?;

    // Generate a file per section
    for (section, section_entries) in &sections {
        let content = renderer.render_section(section, section_entries);
        let filename = format!("{}.org", section);
        let path = options.output_dir.join(&filename);

        std::fs::write(&path, content)?;
        files_written.push(path);
    }

    // Generate index file
    let index_content = renderer.render_index(&sections);
    let index_path = options.output_dir.join("index.org");
    std::fs::write(&index_path, index_content)?;
    files_written.push(index_path);

    Ok(GenerateResult {
        entries_processed: entries.len(),
        files_written,
    })
}

/// Result of documentation generation
#[derive(Debug)]
pub struct GenerateResult {
    /// Number of documented entries processed
    pub entries_processed: usize,
    /// Paths to generated files
    pub files_written: Vec<PathBuf>,
}
