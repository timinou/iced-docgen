//! Widget tree introspection
//!
//! Provides utilities for inspecting the widget tree to understand
//! available actions and UI state.

use iced_test::core::Rectangle;
use std::fmt::Write;

/// A tree representation of the widget hierarchy
#[derive(Debug, Clone)]
pub struct WidgetTree {
    /// The root node of the tree
    pub root: WidgetNode,
}

/// A single node in the widget tree
#[derive(Debug, Clone)]
pub struct WidgetNode {
    /// The kind of widget
    pub kind: WidgetKind,
    /// The widget's bounding rectangle, if known
    pub bounds: Option<Rectangle>,
    /// Text content, if any
    pub text: Option<String>,
    /// Whether this widget is clickable
    pub is_clickable: bool,
    /// Whether this widget can receive text input
    pub is_text_input: bool,
    /// Whether this widget currently has focus
    pub is_focused: bool,
    /// Suggested selector text for this widget
    pub suggested_selector: Option<String>,
    /// Child widgets
    pub children: Vec<WidgetNode>,
}

/// The kind of widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    /// A container widget
    Container,
    /// A row layout
    Row,
    /// A column layout
    Column,
    /// A button
    Button,
    /// A text label
    Text,
    /// A text input
    TextInput,
    /// A scrollable area
    Scrollable,
    /// A pick list / dropdown
    PickList,
    /// A checkbox
    Checkbox,
    /// A toggler / switch
    Toggler,
    /// A slider
    Slider,
    /// An image
    Image,
    /// An SVG
    Svg,
    /// A space / padding element
    Space,
    /// An unknown widget type
    Unknown,
}

impl WidgetKind {
    /// Get a string representation of the widget kind
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Container => "Container",
            Self::Row => "Row",
            Self::Column => "Column",
            Self::Button => "Button",
            Self::Text => "Text",
            Self::TextInput => "TextInput",
            Self::Scrollable => "Scrollable",
            Self::PickList => "PickList",
            Self::Checkbox => "Checkbox",
            Self::Toggler => "Toggler",
            Self::Slider => "Slider",
            Self::Image => "Image",
            Self::Svg => "Svg",
            Self::Space => "Space",
            Self::Unknown => "Unknown",
        }
    }
}

impl WidgetNode {
    /// Create a new widget node
    pub fn new(kind: WidgetKind) -> Self {
        Self {
            kind,
            bounds: None,
            text: None,
            is_clickable: false,
            is_text_input: false,
            is_focused: false,
            suggested_selector: None,
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn with_child(mut self, child: WidgetNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set bounds
    pub fn with_bounds(mut self, bounds: Rectangle) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Set text content
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Mark as clickable
    pub fn clickable(mut self) -> Self {
        self.is_clickable = true;
        self
    }

    /// Mark as text input
    pub fn text_input(mut self) -> Self {
        self.is_text_input = true;
        self
    }

    /// Mark as focused
    pub fn focused(mut self) -> Self {
        self.is_focused = true;
        self
    }

    /// Set suggested selector
    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.suggested_selector = Some(selector.into());
        self
    }
}

impl WidgetTree {
    /// Create a new widget tree with the given root
    pub fn new(root: WidgetNode) -> Self {
        Self { root }
    }

    /// Create an empty tree
    pub fn empty() -> Self {
        Self {
            root: WidgetNode::new(WidgetKind::Container),
        }
    }

    /// Get all clickable elements in the tree
    pub fn clickable(&self) -> Vec<&WidgetNode> {
        let mut result = Vec::new();
        self.collect_clickable(&self.root, &mut result);
        result
    }

    fn collect_clickable<'a>(&'a self, node: &'a WidgetNode, result: &mut Vec<&'a WidgetNode>) {
        if node.is_clickable {
            result.push(node);
        }
        for child in &node.children {
            self.collect_clickable(child, result);
        }
    }

    /// Get all text input elements in the tree
    pub fn text_inputs(&self) -> Vec<&WidgetNode> {
        let mut result = Vec::new();
        self.collect_text_inputs(&self.root, &mut result);
        result
    }

    fn collect_text_inputs<'a>(&'a self, node: &'a WidgetNode, result: &mut Vec<&'a WidgetNode>) {
        if node.is_text_input {
            result.push(node);
        }
        for child in &node.children {
            self.collect_text_inputs(child, result);
        }
    }

    /// Get all visible text in the tree
    pub fn all_text(&self) -> Vec<&str> {
        let mut result = Vec::new();
        self.collect_text(&self.root, &mut result);
        result
    }

    fn collect_text<'a>(&'a self, node: &'a WidgetNode, result: &mut Vec<&'a str>) {
        if let Some(ref text) = node.text {
            result.push(text);
        }
        for child in &node.children {
            self.collect_text(child, result);
        }
    }

    /// Render the tree as ASCII art
    pub fn to_ascii(&self) -> String {
        let mut output = String::new();
        self.render_ascii(&self.root, &mut output, "", true);
        output
    }

    fn render_ascii(&self, node: &WidgetNode, output: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };

        // Build the node description
        let mut desc = node.kind.as_str().to_string();

        if node.is_focused {
            desc.push_str(" [focused]");
        }
        if let Some(ref text) = node.text {
            if text.len() <= 30 {
                write!(desc, " text=\"{}\"", text).unwrap();
            } else {
                write!(desc, " text=\"{}...\"", &text[..27]).unwrap();
            }
        }

        writeln!(output, "{}{}{}", prefix, connector, desc).unwrap();

        // Prepare prefix for children
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        for (i, child) in node.children.iter().enumerate() {
            self.render_ascii(child, output, &child_prefix, i == node.children.len() - 1);
        }
    }

    /// Render the tree as org-mode
    pub fn to_org(&self) -> String {
        let mut output = String::new();
        writeln!(output, "#+begin_src").unwrap();
        output.push_str(&self.to_ascii());
        writeln!(output, "#+end_src").unwrap();
        output
    }
}

impl std::fmt::Display for WidgetTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ascii())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_tree_ascii() {
        let tree = WidgetTree::new(
            WidgetNode::new(WidgetKind::Container)
                .with_child(
                    WidgetNode::new(WidgetKind::TextInput)
                        .text_input()
                        .focused()
                        .with_text("Hello"),
                )
                .with_child(
                    WidgetNode::new(WidgetKind::Row)
                        .with_child(
                            WidgetNode::new(WidgetKind::Button)
                                .clickable()
                                .with_text("OK"),
                        )
                        .with_child(
                            WidgetNode::new(WidgetKind::Button)
                                .clickable()
                                .with_text("Cancel"),
                        ),
                ),
        );

        let ascii = tree.to_ascii();
        assert!(ascii.contains("Container"));
        assert!(ascii.contains("TextInput"));
        assert!(ascii.contains("[focused]"));
        assert!(ascii.contains("Button"));
    }

    #[test]
    fn test_collect_clickable() {
        let tree = WidgetTree::new(
            WidgetNode::new(WidgetKind::Container)
                .with_child(WidgetNode::new(WidgetKind::Text).with_text("Label"))
                .with_child(
                    WidgetNode::new(WidgetKind::Button)
                        .clickable()
                        .with_text("Click me"),
                ),
        );

        let clickable = tree.clickable();
        assert_eq!(clickable.len(), 1);
        assert_eq!(clickable[0].text.as_deref(), Some("Click me"));
    }
}
