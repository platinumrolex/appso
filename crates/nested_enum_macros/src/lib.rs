use proc_macro::{Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree};
use core::str::FromStr;
use std::collections::HashSet;

#[proc_macro]
pub fn ui_blueprint(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut idx = 0;

    // 1. Parse Component Name
    let component_name = match &tokens.get(idx) {
        Some(TokenTree::Ident(id)) => id.to_string(),
        _ => panic!("ui_blueprint! requires a component name (e.g., EngineHeader)"),
    };
    idx += 1;

    // Optional: , zones: RuntimeZone
    let mut zone_enum_name: Option<String> = None;
    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
        if p.as_char() == ',' {
            idx += 1;
            if let Some(TokenTree::Ident(id)) = tokens.get(idx) {
                if id.to_string() == "zones" {
                    idx += 1;
                    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
                        assert_eq!(p.as_char(), ':', "Expected ':' after 'zones' in ui_blueprint!");
                        idx += 1;
                    } else {
                        panic!("Expected ':' after 'zones' in ui_blueprint!");
                    }
                    if let Some(TokenTree::Ident(id)) = tokens.get(idx) {
                        zone_enum_name = Some(id.to_string());
                        idx += 1;
                    } else {
                        panic!("Expected zone enum name after 'zones:' in ui_blueprint!");
                    }
                    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
                        assert_eq!(p.as_char(), ',', "Expected ',' after zone enum name in ui_blueprint!");
                        idx += 1;
                    } else {
                        panic!("Expected ',' after zone enum name in ui_blueprint!");
                    }
                }
            }
        }
    }

    // 2. Extract the body (the { ... } after the name / zones clause)
    let body_group = match &tokens.get(idx) {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g.clone(),
        _ => panic!("Expected {{ ... }} after component name (and optional zones: clause) in ui_blueprint!"),
    };
    idx += 1;
    if idx != tokens.len() {
        panic!("Unexpected trailing tokens in ui_blueprint! macro");
    }

    let body_stream = body_group.stream();

    // 3. Scan for actions
    let mut action_names = HashSet::new();
    scan_for_actions(body_stream.clone(), &mut action_names);
    action_names.insert("None".to_string());
    action_names.insert("Drag".to_string());

    // 3b. Scan for zones inside ui! { ... } invocations
    let mut zones = Vec::new();
    if zone_enum_name.is_some() {
        scan_for_zones(body_stream.clone(), &mut zones);
    }

    let action_enum_name = format!("{}Action", component_name);
    let mut sorted_actions: Vec<_> = action_names.into_iter().collect();
    sorted_actions.sort();

    // 4. Generate zone enum if requested
    let zone_enum_code = if let Some(ref name) = zone_enum_name {
        if zones.is_empty() {
            format!(
                "#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)] pub enum {0} {{ #[default] None }} ",
                name
            )
        } else {
            format!(
                "#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)] pub enum {0} {{ #[default] {1} }} ",
                name,
                zones.join(", ")
            )
        }
    } else {
        String::new()
    };

    // 5. Generate the full code block
    let full_code = format!(
        "{4}\
        #[derive(Clone, Copy, PartialEq, Eq, Debug)] \
        pub enum {0} {{ {1} }} \
        \
        impl wgpu_ui::primitives::UiAction for {0} {{ \
            fn is_interactive(&self) -> bool {{ !matches!(self, {0}::None | {0}::Drag) }} \
        }} \
        \
        impl {2} {{ {3} }}",
        action_enum_name,
        sorted_actions.join(", "),
        component_name,
        body_stream.to_string(),
        zone_enum_code
    );

    TokenStream::from_str(&full_code).expect("Failed to parse generated UI blueprint code")
}

// ---------------------------------------------------------------------------
// Action scanner (unchanged logic, adapted to take &mut HashSet)
// ---------------------------------------------------------------------------
fn scan_for_actions(stream: TokenStream, actions: &mut HashSet<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(id) => {
                let s = id.to_string();
                if s == "action" || s == "toggle_action" {
                    if let Some(TokenTree::Punct(p)) = tokens.get(i + 1) {
                        if p.as_char() == ':' {
                            if let Some(TokenTree::Ident(first)) = tokens.get(i + 2) {
                                let mut name = first.to_string();
                                let mut offset = 3;

                                loop {
                                    if let (
                                        Some(TokenTree::Punct(p1)),
                                        Some(TokenTree::Punct(p2)),
                                        Some(TokenTree::Ident(leaf)),
                                    ) = (
                                        tokens.get(i + offset),
                                        tokens.get(i + offset + 1),
                                        tokens.get(i + offset + 2),
                                    ) {
                                        if p1.as_char() == ':' && p2.as_char() == ':' {
                                            name = leaf.to_string();
                                            offset += 3;
                                            continue;
                                        }
                                    }
                                    break;
                                }

                                let keywords = ["if", "else", "match", "let", "fn", "for", "in"];
                                if !name.ends_with("Action") && !keywords.contains(&name.as_str()) {
                                    actions.insert(name);
                                }
                            }
                        }
                    }
                }
            }
            TokenTree::Group(g) => scan_for_actions(g.stream(), actions),
            _ => {}
        }
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Zone scanner – only looks inside ui! { ... } invocations
// ---------------------------------------------------------------------------
fn scan_for_zones(stream: TokenStream, zones: &mut Vec<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(id) if id.to_string() == "ui" => {
                if let (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g))) =
                    (tokens.get(i + 1), tokens.get(i + 2))
                {
                    if p.as_char() == '!' {
                        scan_ui_macro_for_zones(g.stream(), zones);
                        i += 3;
                        continue;
                    }
                }
            }
            TokenTree::Group(g) => {
                scan_for_zones(g.stream(), zones);
            }
            _ => {}
        }
        i += 1;
    }
}

fn scan_ui_macro_for_zones(stream: TokenStream, zones: &mut Vec<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    for tt in tokens {
        match &tt {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                scan_ui_block_for_zones(g.stream(), zones);
            }
            TokenTree::Group(g) => {
                scan_ui_macro_for_zones(g.stream(), zones);
            }
            _ => {}
        }
    }
}

fn scan_ui_block_for_zones(stream: TokenStream, zones: &mut Vec<String>) {
    let keywords = ["if", "else", "match", "for", "while"]; // Expanded list

    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Ident(id) = &tokens[i] {
            let name = id.to_string();
            
            // CHECK: Is this an Ident followed by a { ... } block?
            if let Some(TokenTree::Group(g)) = tokens.get(i + 1) {
                if g.delimiter() == Delimiter::Brace {
                    
                    // 1. Skip if it's a field type (e.g. title: CustomTitle { ... })
                    let is_field_type = i > 0 && matches!(
                        tokens.get(i - 1),
                        Some(TokenTree::Punct(p)) if p.as_char() == ':'
                    );

                    // 2. NEW CHECK: Skip if it's preceded by 'if' (e.g. if is_maximized { ... })
                    let is_conditional = i > 0 && matches!(
                        tokens.get(i - 1),
                        Some(TokenTree::Ident(prev_id)) if prev_id.to_string() == "if"
                    );

                    if !is_field_type && !is_conditional && !keywords.contains(&name.as_str()) && !zones.contains(&name) {
                        zones.push(name);
                    }
                    
                    scan_ui_block_for_zones(g.stream(), zones);
                    i += 1; 
                }
            }
        }
        i += 1;
    }
}