//! Org-mode rendering for documented items

use crate::GenerateOptions;
use crate::registry::*;
use std::collections::HashMap;
use std::fmt::Write;

/// Renders documentation entries to org-mode format
pub struct OrgRenderer<'a> {
    options: &'a GenerateOptions,
}

impl<'a> OrgRenderer<'a> {
    pub fn new(options: &'a GenerateOptions) -> Self {
        Self { options }
    }

    /// Render an index file linking to all sections
    pub fn render_index(&self, sections: &HashMap<&str, Vec<&DocEntry>>) -> String {
        let mut output = String::new();

        writeln!(
            output,
            "#+TITLE: {} Documentation",
            self.options.project_name
        )
        .unwrap();
        writeln!(output, "#+DATE: {}", chrono_date()).unwrap();
        writeln!(output).unwrap();
        writeln!(output, "* Contents").unwrap();
        writeln!(output).unwrap();

        let mut section_names: Vec<&&str> = sections.keys().collect();
        section_names.sort();

        for section in section_names {
            let entries = &sections[section];
            let title = section_title(section);
            writeln!(
                output,
                "- [[file:{}.org][{}]] ({} items)",
                section,
                title,
                entries.len()
            )
            .unwrap();
        }

        output
    }

    /// Render a section file
    pub fn render_section(&self, section: &str, entries: &[&DocEntry]) -> String {
        let mut output = String::new();

        let title = section_title(section);
        writeln!(output, "#+TITLE: {} - {}", self.options.project_name, title).unwrap();
        writeln!(output, "#+DATE: {}", chrono_date()).unwrap();
        writeln!(output).unwrap();

        for entry in entries {
            self.render_entry(&mut output, entry);
        }

        output
    }

    /// Render a single documentation entry
    fn render_entry(&self, output: &mut String, entry: &DocEntry) {
        // Header with properties
        writeln!(output, "* {}", entry.title).unwrap();
        writeln!(output, ":PROPERTIES:").unwrap();
        writeln!(output, ":ID: {}", entry.id).unwrap();
        writeln!(
            output,
            ":SOURCE: [[file:{}::{}][{}:{}]]",
            entry.source_file, entry.id, entry.source_file, entry.source_line
        )
        .unwrap();
        if !entry.tags.is_empty() {
            writeln!(output, ":TAGS: {}", entry.tags.join(" ")).unwrap();
        }
        writeln!(output, ":END:").unwrap();
        writeln!(output).unwrap();

        // Description
        if !entry.description.is_empty() {
            writeln!(output, "{}", entry.description).unwrap();
            writeln!(output).unwrap();
        }

        // Type-specific content
        match &entry.metadata {
            DocMetadata::Screenshot(meta) => {
                self.render_screenshot_meta(output, meta);
            }
            DocMetadata::Usecase(meta) => {
                self.render_usecase_meta(output, meta);
            }
            DocMetadata::Workflow(meta) => {
                self.render_workflow_meta(output, entry, meta);
            }
            DocMetadata::State(meta) => {
                self.render_state_meta(output, meta);
            }
            DocMetadata::IceTest(meta) => {
                self.render_ice_test_meta(output, meta);
            }
            DocMetadata::Scenario(meta) => {
                self.render_scenario_meta(output, meta);
            }
            DocMetadata::UserStoryMeta(meta) => {
                self.render_user_story_meta(output, meta);
            }
            DocMetadata::None => {}
        }

        // Cross-references
        if !entry.links_to.is_empty() {
            writeln!(output, "*Related:* {}", entry.links_to.join(", ")).unwrap();
            writeln!(output).unwrap();
        }

        if !entry.see_also.is_empty() {
            writeln!(output, "*See also:* {}", entry.see_also.join(", ")).unwrap();
            writeln!(output).unwrap();
        }
    }

    fn render_screenshot_meta(&self, output: &mut String, meta: &ScreenshotMeta) {
        if meta.scenario_names.is_empty() {
            // Single screenshot
            let path = format!(
                "{}/{}.png",
                self.options.screenshots_dir.display(),
                meta.name
            );
            writeln!(output, "[[file:{}]]", path).unwrap();
            if !meta.caption.is_empty() {
                writeln!(output).unwrap();
                writeln!(output, "{}", meta.caption).unwrap();
            }
        } else {
            // Multiple scenarios
            writeln!(output, "** Screenshots").unwrap();
            writeln!(output).unwrap();

            for scenario_name in meta.scenario_names.iter() {
                let path = format!(
                    "{}/{}-{}.png",
                    self.options.screenshots_dir.display(),
                    meta.name,
                    scenario_name
                );
                writeln!(output, "*** {}", scenario_name).unwrap();
                writeln!(output, "[[file:{}]]", path).unwrap();
                writeln!(output).unwrap();
            }
        }
        writeln!(output).unwrap();
    }

    fn render_usecase_meta(&self, output: &mut String, meta: &UsecaseMeta) {
        writeln!(output, "** Actor").unwrap();
        writeln!(output, "{}", meta.actor).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "** Goal").unwrap();
        writeln!(output, "{}", meta.goal).unwrap();
        writeln!(output).unwrap();

        if !meta.preconditions.is_empty() {
            writeln!(output, "** Preconditions").unwrap();
            for pre in meta.preconditions {
                writeln!(output, "- {}", pre).unwrap();
            }
            writeln!(output).unwrap();
        }

        writeln!(output, "** Steps").unwrap();
        for (i, step) in meta.steps.iter().enumerate() {
            writeln!(output, "{}. {}", i + 1, step).unwrap();
        }
        writeln!(output).unwrap();

        if !meta.postconditions.is_empty() {
            writeln!(output, "** Postconditions").unwrap();
            for post in meta.postconditions {
                writeln!(output, "- {}", post).unwrap();
            }
            writeln!(output).unwrap();
        }
    }

    fn render_workflow_meta(&self, output: &mut String, entry: &DocEntry, meta: &WorkflowMeta) {
        writeln!(output, ":PERSONA: {}", meta.persona).unwrap();
        writeln!(output).unwrap();

        if !entry.description.is_empty() {
            writeln!(output, "{}", entry.description).unwrap();
            writeln!(output).unwrap();
        }

        for i in 0..meta.step_views.len() {
            let view = meta.step_views.get(i).unwrap_or(&"");
            let action = meta.step_actions.get(i).unwrap_or(&"");
            let screenshot = meta.step_screenshots.get(i).unwrap_or(&"");
            let description = meta.step_descriptions.get(i).unwrap_or(&"");

            writeln!(output, "** Step {}: {}", i + 1, action).unwrap();

            if !screenshot.is_empty() {
                let path = format!(
                    "{}/{}.png",
                    self.options.screenshots_dir.display(),
                    screenshot
                );
                writeln!(output, "[[file:{}]]", path).unwrap();
                writeln!(output).unwrap();
            }

            if !description.is_empty() {
                writeln!(output, "{}", description).unwrap();
                writeln!(output).unwrap();
            }

            writeln!(output, "- *View:* {}", view).unwrap();
            writeln!(output, "- *Action:* {}", action).unwrap();
            writeln!(output).unwrap();
        }

        if !meta.outcomes.is_empty() {
            writeln!(output, "** Outcomes").unwrap();
            for outcome in meta.outcomes {
                writeln!(output, "- {}", outcome).unwrap();
            }
            writeln!(output).unwrap();
        }
    }

    fn render_state_meta(&self, output: &mut String, meta: &StateMeta) {
        // Mermaid state diagram
        writeln!(output, "** State Diagram").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "#+begin_src mermaid").unwrap();
        writeln!(output, "stateDiagram-v2").unwrap();
        writeln!(output, "    [*] --> {}", meta.initial).unwrap();

        for (i, state_name) in meta.state_names.iter().enumerate() {
            let transitions = meta.state_transitions.get(i).unwrap_or(&"");
            if !transitions.is_empty() {
                for target in transitions.split(',') {
                    let target = target.trim();
                    if !target.is_empty() {
                        writeln!(output, "    {} --> {}", state_name, target).unwrap();
                    }
                }
            }
        }

        for terminal in meta.terminal {
            writeln!(output, "    {} --> [*]", terminal).unwrap();
        }

        writeln!(output, "#+end_src").unwrap();
        writeln!(output).unwrap();

        // State table
        writeln!(output, "** States").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| State | Description | Color | Transitions To |").unwrap();
        writeln!(output, "|-------+-------------+-------+----------------|").unwrap();

        for (i, state_name) in meta.state_names.iter().enumerate() {
            let desc = meta.state_descriptions.get(i).unwrap_or(&"");
            let color = meta.state_colors.get(i).unwrap_or(&"");
            let transitions = meta.state_transitions.get(i).unwrap_or(&"");

            let trans_display = if meta.terminal.contains(state_name) {
                "(terminal)".to_string()
            } else if transitions.is_empty() {
                "-".to_string()
            } else {
                transitions.replace(",", ", ")
            };

            writeln!(
                output,
                "| {} | {} | {} | {} |",
                state_name, desc, color, trans_display
            )
            .unwrap();
        }
        writeln!(output).unwrap();
    }

    fn render_ice_test_meta(&self, output: &mut String, meta: &IceTestMeta) {
        // Setup section
        writeln!(output, "** Setup").unwrap();
        if let Some((w, h)) = meta.viewport {
            writeln!(output, "- *Viewport:* {}x{}", w, h).unwrap();
        }
        writeln!(output, "- *Mode:* {}", meta.mode).unwrap();
        if let Some(preset) = &meta.preset {
            writeln!(output, "- *Preset:* {}", preset).unwrap();
        }
        writeln!(output).unwrap();

        // Steps section with numbered list
        if !meta.instructions.is_empty() {
            writeln!(output, "** Steps").unwrap();
            for (i, instr) in meta.instructions.iter().enumerate() {
                let step_text = match instr.kind.as_str() {
                    "click" => format!("=click= \"{}\"", instr.target),
                    "type" => format!("=type= \"{}\"", instr.value.as_deref().unwrap_or("")),
                    "expect" => format!("=expect= \"{}\"", instr.target),
                    "tap" => format!("=tap= {}", instr.target),
                    "screenshot" => format!("=screenshot= \"{}\"", instr.target),
                    "wait" => format!("=wait= {} ms", instr.value.as_deref().unwrap_or("0")),
                    _ => format!("={}: {}=", instr.kind, instr.target),
                };
                writeln!(output, "{}. {}", i + 1, step_text).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Mermaid sequence diagram
        if !meta.instructions.is_empty() {
            writeln!(output, "** Sequence Diagram").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "#+begin_src mermaid").unwrap();
            writeln!(output, "sequenceDiagram").unwrap();
            writeln!(output, "    participant User").unwrap();
            writeln!(output, "    participant App").unwrap();

            for instr in &meta.instructions {
                match instr.kind.as_str() {
                    "click" => {
                        writeln!(output, "    User->>App: click \"{}\"", instr.target).unwrap();
                    }
                    "type" => {
                        writeln!(
                            output,
                            "    User->>App: type \"{}\"",
                            instr.value.as_deref().unwrap_or("")
                        )
                        .unwrap();
                    }
                    "expect" => {
                        writeln!(output, "    App-->>User: shows \"{}\"", instr.target).unwrap();
                    }
                    "tap" => {
                        writeln!(output, "    User->>App: tap {}", instr.target).unwrap();
                    }
                    "screenshot" => {
                        writeln!(output, "    Note over App: screenshot \"{}\"", instr.target)
                            .unwrap();
                    }
                    "wait" => {
                        writeln!(
                            output,
                            "    Note over User,App: wait {} ms",
                            instr.value.as_deref().unwrap_or("0")
                        )
                        .unwrap();
                    }
                    _ => {
                        writeln!(output, "    User->>App: {} {}", instr.kind, instr.target)
                            .unwrap();
                    }
                }
            }

            writeln!(output, "#+end_src").unwrap();
            writeln!(output).unwrap();
        }

        // Instructions table
        if !meta.instructions.is_empty() {
            writeln!(output, "** Instructions Table").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "| Step | Action | Target | Value |").unwrap();
            writeln!(output, "|------+--------+--------+-------|").unwrap();

            for (i, instr) in meta.instructions.iter().enumerate() {
                let target = if instr.target.is_empty() {
                    "-".to_string()
                } else {
                    format!("\"{}\"", instr.target)
                };
                let value = instr.value.as_ref().map_or("-".to_string(), |v| {
                    if v.is_empty() {
                        "-".to_string()
                    } else {
                        format!("\"{}\"", v)
                    }
                });
                writeln!(
                    output,
                    "| {} | {} | {} | {} |",
                    i + 1,
                    instr.kind,
                    target,
                    value
                )
                .unwrap();
            }
            writeln!(output).unwrap();
        }
    }

    fn render_scenario_meta(&self, output: &mut String, meta: &ScenarioMeta) {
        // Preconditions
        if !meta.preconditions.is_empty() {
            writeln!(output, "** Preconditions").unwrap();
            for pre in meta.preconditions {
                writeln!(output, "- {}", pre).unwrap();
            }
            writeln!(output).unwrap();
        }
    }

    fn render_user_story_meta(&self, output: &mut String, meta: &UserStoryMeta) {
        // Actor and goal
        writeln!(output, "** Actor").unwrap();
        writeln!(output, "{}", meta.actor).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "** Goal").unwrap();
        writeln!(output, "{}", meta.goal).unwrap();
        writeln!(output).unwrap();

        // Preconditions
        if !meta.preconditions.is_empty() {
            writeln!(output, "** Preconditions").unwrap();
            for pre in meta.preconditions {
                writeln!(output, "- {}", pre).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Outcomes
        if !meta.outcomes.is_empty() {
            writeln!(output, "** Expected Outcomes").unwrap();
            for outcome in meta.outcomes {
                writeln!(output, "- [ ] {}", outcome).unwrap();
            }
            writeln!(output).unwrap();
        }
    }
}

/// Get a human-readable title for a section
fn section_title(section: &str) -> &str {
    match section {
        "views" => "Views",
        "models" => "Models",
        "usecases" => "Use Cases",
        "workflows" => "Workflows",
        "screenshots" => "Screenshots",
        "tests" => "End-to-End Tests",
        "default" => "Documentation",
        _ => section,
    }
}

/// Get current date in YYYY-MM-DD format
fn chrono_date() -> String {
    // Simple date without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple calculation (not handling leap years perfectly but good enough)
    let days = secs / 86400;
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for (i, &dim) in days_in_months.iter().enumerate() {
        if remaining_days < dim as u64 {
            month = i + 1;
            break;
        }
        remaining_days -= dim as u64;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_title() {
        assert_eq!(section_title("views"), "Views");
        assert_eq!(section_title("unknown"), "unknown");
    }

    #[test]
    fn test_chrono_date_format() {
        let date = chrono_date();
        assert!(date.len() == 10);
        assert!(date.chars().nth(4) == Some('-'));
        assert!(date.chars().nth(7) == Some('-'));
    }

    #[test]
    fn test_render_empty_section() {
        let options = GenerateOptions::default();
        let renderer = OrgRenderer::new(&options);
        let output = renderer.render_section("views", &[]);

        assert!(output.contains("#+TITLE:"));
        assert!(output.contains("Views"));
    }
}
