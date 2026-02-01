//! Parser and discovery for .ice end-to-end test files
//!
//! This module provides utilities to discover and parse .ice test files,
//! converting them into documentation entries.

use crate::registry::{IceInstruction, IceTestMeta};
use std::fs;
use std::path::{Path, PathBuf};

/// Error type for ice file operations
#[derive(Debug)]
pub enum IceError {
    /// File system error
    Io(std::io::Error),
    /// Parse error from iced_test
    Parse(String),
}

impl std::fmt::Display for IceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IceError::Io(e) => write!(f, "IO error: {}", e),
            IceError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for IceError {}

impl From<std::io::Error> for IceError {
    fn from(e: std::io::Error) -> Self {
        IceError::Io(e)
    }
}

/// Discover all .ice files in a directory (recursively)
pub fn discover_ice_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return files;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(discover_ice_files(&path));
            } else if path.extension().map_or(false, |e| e == "ice") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

/// Parse a single .ice file into IceTestMeta
///
/// The .ice file format is:
/// ```text
/// viewport: WIDTHxHEIGHT
/// mode: Immediate|Patient
/// preset: PresetName
/// -----
/// click "selector"
/// type "text"
/// expect "text"
/// tap KeyName
/// screenshot "name"
/// wait milliseconds
/// ```
pub fn parse_ice_file(path: &Path) -> Result<IceTestMeta, IceError> {
    let content = fs::read_to_string(path)?;
    parse_ice_content(&content, path)
}

/// Parse .ice content string into IceTestMeta
pub fn parse_ice_content(content: &str, path: &Path) -> Result<IceTestMeta, IceError> {
    let mut viewport = None;
    let mut mode = String::from("Immediate");
    let mut preset = None;
    let mut instructions = Vec::new();
    let mut in_instructions = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Separator between metadata and instructions
        if line.starts_with("-----") || line.starts_with("---") {
            in_instructions = true;
            continue;
        }

        if !in_instructions {
            // Parse metadata
            if let Some(value) = line.strip_prefix("viewport:") {
                let value = value.trim();
                if let Some((w, h)) = parse_viewport(value) {
                    viewport = Some((w, h));
                }
            } else if let Some(value) = line.strip_prefix("mode:") {
                mode = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("preset:") {
                preset = Some(value.trim().to_string());
            }
        } else {
            // Parse instructions
            if let Some(instr) = parse_instruction(line) {
                instructions.push(instr);
            }
        }
    }

    Ok(IceTestMeta {
        file_path: path.to_string_lossy().to_string(),
        viewport,
        mode,
        preset,
        instructions,
    })
}

/// Parse viewport string like "800x600" into (width, height)
fn parse_viewport(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse().ok()?;
        let h = parts[1].trim().parse().ok()?;
        Some((w, h))
    } else {
        None
    }
}

/// Parse a single instruction line
fn parse_instruction(line: &str) -> Option<IceInstruction> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Split into command and rest
    let (kind, rest) = if let Some(idx) = line.find(char::is_whitespace) {
        (&line[..idx], line[idx..].trim())
    } else {
        (line, "")
    };

    let kind = kind.to_lowercase();

    match kind.as_str() {
        "click" => Some(IceInstruction {
            kind: "click".to_string(),
            target: unquote(rest),
            value: None,
        }),
        "type" => Some(IceInstruction {
            kind: "type".to_string(),
            target: String::new(),
            value: Some(unquote(rest)),
        }),
        "expect" => Some(IceInstruction {
            kind: "expect".to_string(),
            target: unquote(rest),
            value: None,
        }),
        "tap" => Some(IceInstruction {
            kind: "tap".to_string(),
            target: rest.to_string(),
            value: None,
        }),
        "screenshot" => Some(IceInstruction {
            kind: "screenshot".to_string(),
            target: unquote(rest),
            value: None,
        }),
        "wait" => Some(IceInstruction {
            kind: "wait".to_string(),
            target: String::new(),
            value: Some(rest.to_string()),
        }),
        _ => Some(IceInstruction {
            kind: kind.to_string(),
            target: rest.to_string(),
            value: None,
        }),
    }
}

/// Remove surrounding quotes from a string
fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Get the test name from a file path (filename without extension)
pub fn test_name_from_path(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_viewport() {
        assert_eq!(parse_viewport("800x600"), Some((800, 600)));
        assert_eq!(parse_viewport("1024x768"), Some((1024, 768)));
        assert_eq!(parse_viewport("invalid"), None);
    }

    #[test]
    fn test_parse_instruction_click() {
        let instr = parse_instruction(r#"click "Submit""#).unwrap();
        assert_eq!(instr.kind, "click");
        assert_eq!(instr.target, "Submit");
    }

    #[test]
    fn test_parse_instruction_type() {
        let instr = parse_instruction(r#"type "Hello World""#).unwrap();
        assert_eq!(instr.kind, "type");
        assert_eq!(instr.value, Some("Hello World".to_string()));
    }

    #[test]
    fn test_parse_instruction_tap() {
        let instr = parse_instruction("tap Enter").unwrap();
        assert_eq!(instr.kind, "tap");
        assert_eq!(instr.target, "Enter");
    }

    #[test]
    fn test_parse_ice_content() {
        let content = r#"
viewport: 800x600
mode: Immediate
preset: Empty
-----
click "New Task"
type "Buy groceries"
tap Enter
expect "Buy groceries"
screenshot "task_created"
"#;
        let meta = parse_ice_content(content, Path::new("test.ice")).unwrap();

        assert_eq!(meta.viewport, Some((800, 600)));
        assert_eq!(meta.mode, "Immediate");
        assert_eq!(meta.preset, Some("Empty".to_string()));
        assert_eq!(meta.instructions.len(), 5);
        assert_eq!(meta.instructions[0].kind, "click");
        assert_eq!(meta.instructions[0].target, "New Task");
        assert_eq!(meta.instructions[4].kind, "screenshot");
    }

    #[test]
    fn test_unquote() {
        assert_eq!(unquote(r#""hello""#), "hello");
        assert_eq!(unquote("'hello'"), "hello");
        assert_eq!(unquote("hello"), "hello");
    }

    #[test]
    fn test_test_name_from_path() {
        assert_eq!(
            test_name_from_path(Path::new("tests/ice/create_task.ice")),
            "create_task"
        );
        assert_eq!(test_name_from_path(Path::new("my_test.ice")), "my_test");
    }
}
