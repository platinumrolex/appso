use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use core::str::FromStr;
use std::collections::HashSet;

#[proc_macro]
pub fn ui_blueprint(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter().peekable();
    
    // 1. Parse Component Name
    let component_name = match tokens.next() {
        Some(TokenTree::Ident(id)) => id,
        _ => panic!("ui_blueprint! requires a component name (e.g., EngineHeader)"),
    };

    // 2. Extract the body (the { ... } after the name)
    let body_group = match tokens.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
        _ => panic!("Expected {{ ... }} after component name"),
    };

    let body_stream = body_group.stream();

    // 3. Scan for actions
    let mut action_names = HashSet::new();
    scan_for_actions(body_stream.clone(), &mut action_names);

    // Baseline required actions
    action_names.insert("None".to_string());
    action_names.insert("Drag".to_string());

    let action_enum_name = format!("{}Action", component_name);
    let mut sorted_actions: Vec<_> = action_names.into_iter().collect();
    sorted_actions.sort();

    // 4. Generate the code as a single valid block to avoid from_str panics
    let full_code = format!(
        "#[derive(Clone, Copy, PartialEq, Eq, Debug)] \
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
        body_stream.to_string()
    );

    TokenStream::from_str(&full_code).expect("Failed to parse generated UI blueprint code")
}

fn scan_for_actions(stream: TokenStream, actions: &mut HashSet<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(id) => {
                let s = id.to_string();
                // 1. Only look for property assignments: "action:" or "toggle_action:"
                if s == "action" || s == "toggle_action" {
                    // Check if the next token is a colon ':'
                    if let Some(TokenTree::Punct(p)) = tokens.get(i + 1) {
                        if p.as_char() == ':' {
                            // 2. Look for the variant identifier at i + 2
                            if let Some(TokenTree::Ident(first)) = tokens.get(i + 2) {
                                let mut name = first.to_string();
                                let mut offset = 3;

                                // 3. Robust Path Resolution: Handle "Enum::Variant"
                                loop {
                                    if let (Some(TokenTree::Punct(p1)), Some(TokenTree::Punct(p2)), Some(TokenTree::Ident(leaf))) = 
                                        (tokens.get(i + offset), tokens.get(i + offset + 1), tokens.get(i + offset + 2)) 
                                    {
                                        if p1.as_char() == ':' && p2.as_char() == ':' {
                                            name = leaf.to_string();
                                            offset += 3;
                                            continue;
                                        }
                                    }
                                    break;
                                }

                                // 4. Safety: Ignore the Enum name itself and Rust keywords
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