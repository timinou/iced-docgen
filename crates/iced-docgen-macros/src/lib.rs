//! Procedural macros for iced-docgen
//!
//! This crate provides attribute macros for annotating Iced application code
//! to generate org-mode documentation automatically.
#![allow(clippy::collapsible_if)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, ItemEnum, ItemFn, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Key-value pair for macro attributes
struct KeyValue {
    key: Ident,
    value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(KeyValue { key, value })
    }
}

/// Parsed attributes for #[documented]
struct DocumentedArgs {
    title: Option<String>,
    section: Option<String>,
    tags: Vec<String>,
    links_to: Vec<String>,
    see_also: Vec<String>,
}

impl Parse for DocumentedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = DocumentedArgs {
            title: None,
            section: None,
            tags: vec![],
            links_to: vec![],
            see_also: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = Some(s.value());
                        }
                    }
                }
                "section" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.section = Some(s.value());
                        }
                    }
                }
                "tags" => {
                    args.tags = parse_string_array(&kv.value);
                }
                "links_to" => {
                    args.links_to = parse_string_array(&kv.value);
                }
                "see_also" => {
                    args.see_also = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

fn parse_string_array(expr: &Expr) -> Vec<String> {
    let mut result = vec![];
    if let Expr::Array(arr) = expr {
        for elem in &arr.elems {
            if let Expr::Lit(lit) = elem {
                if let syn::Lit::Str(s) = &lit.lit {
                    result.push(s.value());
                }
            }
        }
    }
    result
}

/// Marks an item for documentation generation.
///
/// # Example
///
/// ```ignore
/// #[documented(
///     title = "Task List View",
///     section = "views",
///     tags = ["ui", "tasks"],
///     links_to = ["AppState", "Task"],
///     see_also = ["view_kanban"]
/// )]
/// pub fn view_task_list(state: &AppState) -> Element<Message> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn documented(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as DocumentedArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let title = args.title.unwrap_or_else(|| fn_name_str.clone());
    let section = args.section.unwrap_or_else(|| "default".to_string());
    let tags: Vec<&str> = args.tags.iter().map(|s| s.as_str()).collect();
    let links_to: Vec<&str> = args.links_to.iter().map(|s| s.as_str()).collect();
    let see_also: Vec<&str> = args.see_also.iter().map(|s| s.as_str()).collect();

    // Extract doc comments from the function
    let doc_comments: Vec<String> = input
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                attr.meta.require_name_value().ok().and_then(|nv| {
                    if let Expr::Lit(lit) = &nv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                    None
                })
            } else {
                None
            }
        })
        .collect();

    let description = doc_comments.join("\n");

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::DocEntry {
                kind: ::iced_docgen::DocKind::Function,
                id: #fn_name_str,
                title: #title,
                section: #section,
                description: #description,
                source_file: file!(),
                source_line: line!(),
                tags: &[#(#tags),*],
                links_to: &[#(#links_to),*],
                see_also: &[#(#see_also),*],
                metadata: ::iced_docgen::DocMetadata::None,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parsed attributes for #[screenshot]
struct ScreenshotArgs {
    name: String,
    theme: String,
    scenarios: Vec<ScreenshotScenario>,
    caption: Option<String>,
}

struct ScreenshotScenario {
    name: String,
    state_expr: String,
}

impl Parse for ScreenshotArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = ScreenshotArgs {
            name: String::new(),
            theme: "Light".to_string(),
            scenarios: vec![],
            caption: None,
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "name" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.name = s.value();
                        }
                    }
                }
                "theme" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.theme = s.value();
                        }
                    }
                }
                "caption" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.caption = Some(s.value());
                        }
                    }
                }
                "scenarios" => {
                    args.scenarios = parse_scenarios(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

fn parse_scenarios(expr: &Expr) -> Vec<ScreenshotScenario> {
    let mut scenarios = vec![];

    if let Expr::Array(arr) = expr {
        for elem in &arr.elems {
            // Support tuple syntax: ("name", "state_expr")
            if let Expr::Tuple(tuple) = elem {
                if tuple.elems.len() >= 2 {
                    let mut name = String::new();
                    let mut state_expr = String::new();

                    if let Some(Expr::Lit(lit)) = tuple.elems.first() {
                        if let syn::Lit::Str(s) = &lit.lit {
                            name = s.value();
                        }
                    }
                    if let Some(Expr::Lit(lit)) = tuple.elems.get(1) {
                        if let syn::Lit::Str(s) = &lit.lit {
                            state_expr = s.value();
                        }
                    }

                    if !name.is_empty() && !state_expr.is_empty() {
                        scenarios.push(ScreenshotScenario { name, state_expr });
                    }
                }
            }
        }
    }

    scenarios
}

/// Marks a view function for automatic screenshot generation.
///
/// # Example
///
/// ```ignore
/// #[screenshot(
///     name = "kanban",
///     theme = "Light",
///     scenarios = [
///         ("default", "create_mock_state()"),
///         ("empty", "AppState::default()")
///     ],
///     caption = "Kanban board view"
/// )]
/// pub fn view_kanban(state: &AppState) -> Element<Message> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn screenshot(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ScreenshotArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let screenshot_name = if args.name.is_empty() {
        fn_name_str.clone()
    } else {
        args.name.clone()
    };

    let theme = &args.theme;
    let caption = args.caption.as_deref().unwrap_or("");

    // Build scenario data for registry
    let scenario_names: Vec<&str> = args.scenarios.iter().map(|s| s.name.as_str()).collect();
    let scenario_states: Vec<&str> = args
        .scenarios
        .iter()
        .map(|s| s.state_expr.as_str())
        .collect();

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::DocEntry {
                kind: ::iced_docgen::DocKind::Function,
                id: #fn_name_str,
                title: #fn_name_str,
                section: "screenshots",
                description: #caption,
                source_file: file!(),
                source_line: line!(),
                tags: &[],
                links_to: &[],
                see_also: &[],
                metadata: ::iced_docgen::DocMetadata::Screenshot(::iced_docgen::ScreenshotMeta {
                    name: #screenshot_name,
                    theme: #theme,
                    scenario_names: &[#(#scenario_names),*],
                    scenario_states: &[#(#scenario_states),*],
                    caption: #caption,
                }),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parsed attributes for #[usecase]
struct UsecaseArgs {
    title: String,
    actor: String,
    goal: String,
    preconditions: Vec<String>,
    steps: Vec<String>,
    postconditions: Vec<String>,
    tags: Vec<String>,
}

impl Parse for UsecaseArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = UsecaseArgs {
            title: String::new(),
            actor: String::new(),
            goal: String::new(),
            preconditions: vec![],
            steps: vec![],
            postconditions: vec![],
            tags: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = s.value();
                        }
                    }
                }
                "actor" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.actor = s.value();
                        }
                    }
                }
                "goal" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.goal = s.value();
                        }
                    }
                }
                "preconditions" => {
                    args.preconditions = parse_string_array(&kv.value);
                }
                "steps" => {
                    args.steps = parse_string_array(&kv.value);
                }
                "postconditions" => {
                    args.postconditions = parse_string_array(&kv.value);
                }
                "tags" => {
                    args.tags = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Documents a user story that doubles as a test.
///
/// # Example
///
/// ```ignore
/// #[usecase(
///     title = "Complete Task from Kanban",
///     actor = "Admin",
///     goal = "Mark a task complete by clicking it",
///     preconditions = ["Logged in", "Task exists"],
///     steps = ["Navigate to Kanban", "Click task card"],
///     postconditions = ["Task status is Done"]
/// )]
/// #[test]
/// fn test_complete_task() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn usecase(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UsecaseArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let title = &args.title;
    let actor = &args.actor;
    let goal = &args.goal;
    let preconditions: Vec<&str> = args.preconditions.iter().map(|s| s.as_str()).collect();
    let steps: Vec<&str> = args.steps.iter().map(|s| s.as_str()).collect();
    let postconditions: Vec<&str> = args.postconditions.iter().map(|s| s.as_str()).collect();
    let tags: Vec<&str> = args.tags.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::DocEntry {
                kind: ::iced_docgen::DocKind::Usecase,
                id: #fn_name_str,
                title: #title,
                section: "usecases",
                description: #goal,
                source_file: file!(),
                source_line: line!(),
                tags: &[#(#tags),*],
                links_to: &[],
                see_also: &[],
                metadata: ::iced_docgen::DocMetadata::Usecase(::iced_docgen::UsecaseMeta {
                    actor: #actor,
                    goal: #goal,
                    preconditions: &[#(#preconditions),*],
                    steps: &[#(#steps),*],
                    postconditions: &[#(#postconditions),*],
                }),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parsed attributes for #[workflow]
struct WorkflowArgs {
    title: String,
    description: String,
    persona: String,
    steps: Vec<WorkflowStep>,
    outcomes: Vec<String>,
}

struct WorkflowStep {
    view: String,
    action: String,
    screenshot: String,
    description: String,
}

impl Parse for WorkflowArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = WorkflowArgs {
            title: String::new(),
            description: String::new(),
            persona: String::new(),
            steps: vec![],
            outcomes: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = s.value();
                        }
                    }
                }
                "description" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.description = s.value();
                        }
                    }
                }
                "persona" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.persona = s.value();
                        }
                    }
                }
                "steps" => {
                    args.steps = parse_workflow_steps(&kv.value);
                }
                "outcomes" => {
                    args.outcomes = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

fn parse_workflow_steps(expr: &Expr) -> Vec<WorkflowStep> {
    let mut steps = vec![];

    if let Expr::Array(arr) = expr {
        for elem in &arr.elems {
            // Support tuple syntax: ("view", "action", "screenshot", "description")
            if let Expr::Tuple(tuple) = elem {
                let mut view = String::new();
                let mut action = String::new();
                let mut screenshot = String::new();
                let mut description = String::new();

                if let Some(Expr::Lit(lit)) = tuple.elems.first() {
                    if let syn::Lit::Str(s) = &lit.lit {
                        view = s.value();
                    }
                }
                if let Some(Expr::Lit(lit)) = tuple.elems.get(1) {
                    if let syn::Lit::Str(s) = &lit.lit {
                        action = s.value();
                    }
                }
                if let Some(Expr::Lit(lit)) = tuple.elems.get(2) {
                    if let syn::Lit::Str(s) = &lit.lit {
                        screenshot = s.value();
                    }
                }
                if let Some(Expr::Lit(lit)) = tuple.elems.get(3) {
                    if let syn::Lit::Str(s) = &lit.lit {
                        description = s.value();
                    }
                }

                steps.push(WorkflowStep {
                    view,
                    action,
                    screenshot,
                    description,
                });
            }
        }
    }

    steps
}

/// Documents a multi-step user journey.
///
/// # Example
///
/// ```ignore
/// #[workflow(
///     title = "Morning Review",
///     persona = "Project Manager",
///     description = "How admins start their day",
///     steps = [
///         ("TaskList", "Filter by self", "morning_1", "See your tasks"),
///         ("Calendar", "Check deadlines", "morning_2", "What's due today")
///     ],
///     outcomes = ["Admin knows priorities", "Overdue items identified"]
/// )]
/// fn document_morning_workflow() {}
/// ```
#[proc_macro_attribute]
pub fn workflow(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as WorkflowArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let title = &args.title;
    let description = &args.description;
    let persona = &args.persona;
    let outcomes: Vec<&str> = args.outcomes.iter().map(|s| s.as_str()).collect();

    // Flatten workflow steps into parallel arrays for static storage
    let step_views: Vec<&str> = args.steps.iter().map(|s| s.view.as_str()).collect();
    let step_actions: Vec<&str> = args.steps.iter().map(|s| s.action.as_str()).collect();
    let step_screenshots: Vec<&str> = args.steps.iter().map(|s| s.screenshot.as_str()).collect();
    let step_descriptions: Vec<&str> = args.steps.iter().map(|s| s.description.as_str()).collect();

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::DocEntry {
                kind: ::iced_docgen::DocKind::Workflow,
                id: #fn_name_str,
                title: #title,
                section: "workflows",
                description: #description,
                source_file: file!(),
                source_line: line!(),
                tags: &[],
                links_to: &[],
                see_also: &[],
                metadata: ::iced_docgen::DocMetadata::Workflow(::iced_docgen::WorkflowMeta {
                    persona: #persona,
                    step_views: &[#(#step_views),*],
                    step_actions: &[#(#step_actions),*],
                    step_screenshots: &[#(#step_screenshots),*],
                    step_descriptions: &[#(#step_descriptions),*],
                    outcomes: &[#(#outcomes),*],
                }),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parsed attributes for #[state_doc]
struct StateDocArgs {
    title: Option<String>,
    description: Option<String>,
    initial: String,
    terminal: Vec<String>,
}

impl Parse for StateDocArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = StateDocArgs {
            title: None,
            description: None,
            initial: String::new(),
            terminal: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = Some(s.value());
                        }
                    }
                }
                "description" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.description = Some(s.value());
                        }
                    }
                }
                "initial" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.initial = s.value();
                        }
                    }
                }
                "terminal" => {
                    args.terminal = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Parsed attributes for #[state] (variant helper)
struct StateArgs {
    description: Option<String>,
    color: Option<String>,
    transitions_to: Vec<String>,
    terminal: bool,
}

impl Parse for StateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = StateArgs {
            description: None,
            color: None,
            transitions_to: vec![],
            terminal: false,
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "description" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.description = Some(s.value());
                        }
                    }
                }
                "color" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.color = Some(s.value());
                        }
                    }
                }
                "transitions_to" => {
                    args.transitions_to = parse_string_array(&kv.value);
                }
                "terminal" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Bool(b) = &lit.lit {
                            args.terminal = b.value();
                        }
                    }
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Documents a state machine enum with transitions.
///
/// # Example
///
/// ```ignore
/// #[state_doc(initial = "Backlog", terminal = ["Done"])]
/// pub enum TaskStatus {
///     #[state(transitions_to = ["Todo"])]
///     Backlog,
///     #[state(transitions_to = ["InProgress", "Backlog"])]
///     Todo,
///     #[state(terminal = true)]
///     Done,
/// }
/// ```
#[proc_macro_attribute]
pub fn state_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as StateDocArgs);
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let enum_name_str = enum_name.to_string();

    let title = args.title.unwrap_or_else(|| enum_name_str.clone());
    let description = args.description.unwrap_or_default();
    let initial = &args.initial;
    let terminal: Vec<&str> = args.terminal.iter().map(|s| s.as_str()).collect();

    // Extract state info from variants
    let mut state_names: Vec<String> = vec![];
    let mut state_descriptions: Vec<String> = vec![];
    let mut state_colors: Vec<String> = vec![];
    let mut state_transitions: Vec<String> = vec![]; // Comma-separated transitions per state

    for variant in &input.variants {
        let variant_name = variant.ident.to_string();
        state_names.push(variant_name);

        // Find #[state(...)] attribute
        let mut desc = String::new();
        let mut color = String::new();
        let mut transitions = String::new();

        for attr in &variant.attrs {
            if attr.path().is_ident("state") {
                if let Ok(state_args) = attr.parse_args::<StateArgs>() {
                    desc = state_args.description.unwrap_or_default();
                    color = state_args.color.unwrap_or_default();
                    transitions = state_args.transitions_to.join(",");
                }
            }
        }

        state_descriptions.push(desc);
        state_colors.push(color);
        state_transitions.push(transitions);
    }

    let state_names_refs: Vec<&str> = state_names.iter().map(|s| s.as_str()).collect();
    let state_desc_refs: Vec<&str> = state_descriptions.iter().map(|s| s.as_str()).collect();
    let state_color_refs: Vec<&str> = state_colors.iter().map(|s| s.as_str()).collect();
    let state_trans_refs: Vec<&str> = state_transitions.iter().map(|s| s.as_str()).collect();

    // Strip #[state(...)] attributes from output (they're not real Rust attributes)
    let mut clean_input = input.clone();
    for variant in &mut clean_input.variants {
        variant.attrs.retain(|attr| !attr.path().is_ident("state"));
    }

    let expanded = quote! {
        #clean_input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::DocEntry {
                kind: ::iced_docgen::DocKind::Enum,
                id: #enum_name_str,
                title: #title,
                section: "models",
                description: #description,
                source_file: file!(),
                source_line: line!(),
                tags: &["state-machine"],
                links_to: &[],
                see_also: &[],
                metadata: ::iced_docgen::DocMetadata::State(::iced_docgen::StateMeta {
                    initial: #initial,
                    terminal: &[#(#terminal),*],
                    state_names: &[#(#state_names_refs),*],
                    state_descriptions: &[#(#state_desc_refs),*],
                    state_colors: &[#(#state_color_refs),*],
                    state_transitions: &[#(#state_trans_refs),*],
                }),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Helper attribute for state machine variants (used with #[state_doc])
#[proc_macro_attribute]
pub fn state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is a marker attribute that gets parsed by state_doc
    // We just pass the item through unchanged
    item
}

// =============================================================================
// Visual Testing DSL Macros
// =============================================================================

/// Parsed attributes for #[scenario]
struct ScenarioArgs {
    title: String,
    description: String,
    preconditions: Vec<String>,
    tags: Vec<String>,
}

impl Parse for ScenarioArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = ScenarioArgs {
            title: String::new(),
            description: String::new(),
            preconditions: vec![],
            tags: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = s.value();
                        }
                    }
                }
                "description" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.description = s.value();
                        }
                    }
                }
                "preconditions" => {
                    args.preconditions = parse_string_array(&kv.value);
                }
                "tags" => {
                    args.tags = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Documents a test scenario for visual testing.
///
/// This macro registers the test with iced-docgen's registry and generates
/// documentation from test execution.
///
/// # Example
///
/// ```ignore
/// #[scenario(
///     title = "Add New Task",
///     description = "User adds a task via input field",
///     preconditions = ["Empty task list"],
///     tags = ["tasks", "input"]
/// )]
/// #[test]
/// fn test_add_task() {
///     let app = MyApp::new();
///     let mut ctx = TestContext::new(app.view());
///
///     ctx.execute(TestAction::click("input").described_as("Focus input"))?;
///     ctx.execute(TestAction::typewrite("Buy milk"))?;
///     ctx.execute(TestAction::tap(Key::Enter).with_screenshot())?;
///     ctx.execute(TestAction::expect("Buy milk"))?;
/// }
/// ```
#[proc_macro_attribute]
pub fn scenario(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ScenarioArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let title = if args.title.is_empty() {
        fn_name_str.clone()
    } else {
        args.title.clone()
    };
    let description = &args.description;
    let preconditions: Vec<&str> = args.preconditions.iter().map(|s| s.as_str()).collect();
    let tags: Vec<&str> = args.tags.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::TestScenarioEntry {
                id: #fn_name_str,
                title: #title,
                description: #description,
                actor: "User",
                preconditions: &[#(#preconditions),*],
                outcomes: &[],
                tags: &[#(#tags),*],
                source_file: file!(),
                source_line: line!(),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parsed attributes for #[user_story]
struct UserStoryArgs {
    title: String,
    actor: String,
    goal: String,
    preconditions: Vec<String>,
    outcomes: Vec<String>,
    tags: Vec<String>,
}

impl Parse for UserStoryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = UserStoryArgs {
            title: String::new(),
            actor: "User".to_string(),
            goal: String::new(),
            preconditions: vec![],
            outcomes: vec![],
            tags: vec![],
        };

        while !input.is_empty() {
            let kv: KeyValue = input.parse()?;
            let key_str = kv.key.to_string();

            match key_str.as_str() {
                "title" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.title = s.value();
                        }
                    }
                }
                "actor" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.actor = s.value();
                        }
                    }
                }
                "goal" => {
                    if let Expr::Lit(lit) = &kv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            args.goal = s.value();
                        }
                    }
                }
                "preconditions" => {
                    args.preconditions = parse_string_array(&kv.value);
                }
                "outcomes" => {
                    args.outcomes = parse_string_array(&kv.value);
                }
                "tags" => {
                    args.tags = parse_string_array(&kv.value);
                }
                _ => {}
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Documents a multi-step user story for visual testing.
///
/// User stories represent complete user journeys and generate comprehensive
/// documentation including actor, goal, preconditions, and outcomes.
///
/// # Example
///
/// ```ignore
/// #[user_story(
///     title = "Morning Review",
///     actor = "Developer",
///     goal = "Review and prioritize tasks",
///     preconditions = ["Logged in", "Has pending tasks"],
///     outcomes = ["Clear priorities", "Urgent tasks identified"],
///     tags = ["workflow"]
/// )]
/// fn story_morning_review() {
///     let mut ctx = TestContext::new(app.view());
///
///     ctx.step("Open dashboard");
///     ctx.execute(TestAction::click("Dashboard").with_screenshot())?;
///     ctx.execute(TestAction::expect("Task Overview"))?;
///
///     ctx.step("Filter high priority");
///     ctx.execute(TestAction::click("Priority: High").with_screenshot())?;
///
///     // Story documentation generated from ctx.to_story()
/// }
/// ```
#[proc_macro_attribute]
pub fn user_story(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UserStoryArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let title = if args.title.is_empty() {
        fn_name_str.clone()
    } else {
        args.title.clone()
    };
    let actor = &args.actor;
    let goal = &args.goal;
    let preconditions: Vec<&str> = args.preconditions.iter().map(|s| s.as_str()).collect();
    let outcomes: Vec<&str> = args.outcomes.iter().map(|s| s.as_str()).collect();
    let tags: Vec<&str> = args.tags.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        #input

        ::iced_docgen::inventory::submit! {
            ::iced_docgen::TestScenarioEntry {
                id: #fn_name_str,
                title: #title,
                description: #goal,
                actor: #actor,
                preconditions: &[#(#preconditions),*],
                outcomes: &[#(#outcomes),*],
                tags: &[#(#tags),*],
                source_file: file!(),
                source_line: line!(),
            }
        }
    };

    TokenStream::from(expanded)
}
