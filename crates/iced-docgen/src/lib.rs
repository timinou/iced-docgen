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
pub mod ice;

pub use inventory;
pub use registry::*;
pub use render::*;
pub use ice::{discover_ice_files, parse_ice_file, test_name_from_path, IceError};

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
    /// Directory containing .ice test files (optional)
    pub ice_tests_dir: Option<PathBuf>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs"),
            screenshots_dir: PathBuf::from("screenshots"),
            project_name: "Project".to_string(),
            include_ai_export: false,
            ice_tests_dir: None,
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

    // Discover and parse .ice test files if directory provided
    let ice_entries: Vec<DocEntry> = if let Some(ice_dir) = &options.ice_tests_dir {
        discover_ice_files(ice_dir)
            .into_iter()
            .filter_map(|path| {
                parse_ice_file(&path).ok().map(|meta| {
                    let test_name = test_name_from_path(&path);
                    DocEntry {
                        kind: DocKind::IceTest,
                        id: Box::leak(test_name.clone().into_boxed_str()),
                        title: Box::leak(format!("Test: {}", test_name).into_boxed_str()),
                        section: "tests",
                        description: "",
                        source_file: Box::leak(meta.file_path.clone().into_boxed_str()),
                        source_line: 0,
                        tags: &[],
                        links_to: &[],
                        see_also: &[],
                        metadata: DocMetadata::IceTest(meta),
                    }
                })
            })
            .collect()
    } else {
        Vec::new()
    };

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

    // Generate tests section from ice_entries
    if !ice_entries.is_empty() {
        let test_refs: Vec<&DocEntry> = ice_entries.iter().collect();
        let content = renderer.render_section("tests", &test_refs);
        let path = options.output_dir.join("tests.org");
        std::fs::write(&path, content)?;
        files_written.push(path);
    }

    // Generate index file
    let index_content = renderer.render_index(&sections);
    let index_path = options.output_dir.join("index.org");
    std::fs::write(&index_path, index_content)?;
    files_written.push(index_path);

    Ok(GenerateResult {
        entries_processed: entries.len() + ice_entries.len(),
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
