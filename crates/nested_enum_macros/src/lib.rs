use proc_macro::{Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree};
use core::str::FromStr;
use std::collections::HashSet;

#[proc_macro]
pub fn ui_blueprint(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut idx = 0;

    // ---------------------------------------------------------------------
    // Component name
    // ---------------------------------------------------------------------
    let component_name = match &tokens.get(idx) {
        Some(TokenTree::Ident(id)) => id.to_string(),
        _ => panic!("ui_blueprint! requires component name"),
    };
    idx += 1;

    // ---------------------------------------------------------------------
    // Optional zones
    // ---------------------------------------------------------------------
    let mut zone_enum_name: Option<String> = None;

    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
        if p.as_char() == ',' {
            idx += 1;
            if let Some(TokenTree::Ident(id)) = tokens.get(idx) {
                if id.to_string() == "zones" {
                    idx += 1;
                    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
                        assert_eq!(p.as_char(), ':');
                        idx += 1;
                    }
                    if let Some(TokenTree::Ident(id)) = tokens.get(idx) {
                        zone_enum_name = Some(id.to_string());
                        idx += 1;
                    }
                    if let Some(TokenTree::Punct(p)) = tokens.get(idx) {
                        assert_eq!(p.as_char(), ',');
                        idx += 1;
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // Body
    // ---------------------------------------------------------------------
    let body_group = match &tokens.get(idx) {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g.clone(),
        _ => panic!("Expected {{ ... }} body"),
    };

    let body_stream = body_group.stream();

    // ---------------------------------------------------------------------
    // ACTION SCAN
    // ---------------------------------------------------------------------
    let mut action_names = HashSet::new();
    scan_for_actions(body_stream.clone(), &mut action_names);

    action_names.insert("None".to_string());
    action_names.insert("Drag".to_string());

    let action_enum_name = format!("{}Action", component_name);

    let mut sorted_actions: Vec<_> = action_names.into_iter().collect();
    sorted_actions.sort();

    // ---------------------------------------------------------------------
    // ZONE ENUM
    // ---------------------------------------------------------------------
    // ---------------------------------------------------------------------
    // SCAN FOR ZONES
    // ---------------------------------------------------------------------
    let mut zone_names = Vec::new();
    scan_for_zones(body_stream.clone(), &mut zone_names);
    zone_names.sort();
    zone_names.dedup();

    // ---------------------------------------------------------------------
    // ZONE ENUM
    // ---------------------------------------------------------------------
    let zone_enum_code = if let Some(ref name) = zone_enum_name {
        if zone_names.is_empty() {
            // No zones found – emit an empty enum (or omit entirely, but keep for consistency)
            format!(
                "#[derive(PartialEq, Eq, Debug, Clone, Copy)]
                pub enum {name} {{}}"
            )
        } else {
            let variants = zone_names.join(", ");
            format!(
                "#[derive(PartialEq, Eq, Debug, Clone, Copy)]
                pub enum {name} {{ {variants} }}"
            )
        }
    } else {
        String::new()
    };

    // ---------------------------------------------------------------------
    // FINAL OUTPUT
    // ---------------------------------------------------------------------
    let full_code = format!(
        r#"
        {zone_enum_code}

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum {action_enum_name} {{
            {actions},
            ToggleSelector(&'static str),
            CloseSelectors,
        }}

        impl wgpu_ui::primitives::UiAction for {action_enum_name} {{
            fn is_interactive(&self) -> bool {{
                !matches!(self, {action_enum_name}::None | {action_enum_name}::Drag)
            }}
        }}

        impl {component_name} {{

            // -------------------------------------------------------------
            // SELECTOR STATE (framework-owned)
            // -------------------------------------------------------------
            pub fn selector_open(&self, id: &'static str) -> bool {{
                self.__open_selectors.contains(id)
            }}

            pub fn toggle_selector(&mut self, id: &'static str) {{
                if self.__open_selectors.contains(id) {{
                    self.__open_selectors.remove(id);
                }} else {{
                    self.__open_selectors.clear();
                    self.__open_selectors.insert(id);
                }}
            }}

            pub fn close_selector(&mut self, id: &'static str) {{
                self.__open_selectors.remove(id);
            }}

            // -------------------------------------------------------------
            // USER LOGIC
            // -------------------------------------------------------------
            {body}
        }}
        "#,
        zone_enum_code = zone_enum_code,
        action_enum_name = action_enum_name,
        actions = sorted_actions.join(", "),
        component_name = component_name,
        body = body_stream.to_string()
    );

    TokenStream::from_str(&full_code).unwrap()
}

// ---------------------------------------------------------------------------
// Action scanner
// ---------------------------------------------------------------------------
fn scan_for_actions(stream: TokenStream, actions: &mut HashSet<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {

            // -----------------------------------------------------------------
            // EXISTING: action: Something / toggle_action: Something
            // -----------------------------------------------------------------
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
                                        Some(TokenTree::Ident(next)),
                                    ) = (
                                        tokens.get(i + offset),
                                        tokens.get(i + offset + 1),
                                        tokens.get(i + offset + 2),
                                    ) {
                                        if p1.as_char() == ':' && p2.as_char() == ':' {
                                            name = next.to_string();
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

            // -----------------------------------------------------------------
            // NEW: @ToggleSettings OR @Namespace::ToggleSettings
            // -----------------------------------------------------------------
            TokenTree::Punct(p) if p.as_char() == '@' => {
                if let Some(TokenTree::Ident(first)) = tokens.get(i + 1) {
                    let first_name = first.to_string();

                    // 🚫 SKIP INTERNAL MACRO DIRECTIVES
                    let internal = ["to", "parse", "render", "apply"];
                    if internal.contains(&first_name.as_str()) {
                        i += 1;
                        continue;
                    }

                    let mut name = first_name;
                    let mut offset = 2;

                    // handle @Foo::Bar
                    loop {
                        if let (
                            Some(TokenTree::Punct(p1)),
                            Some(TokenTree::Punct(p2)),
                            Some(TokenTree::Ident(next)),
                        ) = (
                            tokens.get(i + offset),
                            tokens.get(i + offset + 1),
                            tokens.get(i + offset + 2),
                        ) {
                            if p1.as_char() == ':' && p2.as_char() == ':' {
                                name = next.to_string();
                                offset += 3;
                                continue;
                            }
                        }
                        break;
                    }

                    let keywords = ["if", "else", "match", "let", "fn", "for", "in"];

                    if !keywords.contains(&name.as_str()) {
                        actions.insert(name);
                    }
                }
            }

            // -----------------------------------------------------------------
            // RECURSE
            // -----------------------------------------------------------------
            TokenTree::Group(g) => {
                scan_for_actions(g.stream(), actions);
            }

            _ => {}
        }

        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Zone scanner – looks inside ui! and section! invocations
// ---------------------------------------------------------------------------
fn scan_for_zones(stream: TokenStream, zones: &mut Vec<String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(id) if id.to_string() == "ui" || id.to_string() == "section" => {
                if let (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g))) =
                    (tokens.get(i + 1), tokens.get(i + 2))
                {
                    if p.as_char() == '!' {
                        scan_stream_for_zones(g.stream(), zones);
                        i += 2;
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

fn scan_stream_for_zones(stream: TokenStream, zones: &mut Vec<String>) {
    let keywords = ["if", "else", "match", "for", "while", "loop", "let", "mut", "fn", "return", "break", "continue"];

    let tokens: Vec<_> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Ident(id) = &tokens[i] {
            let name = id.to_string();
            
            // CHECK: Is this an Ident followed by a { ... } block?
            if let Some(TokenTree::Group(g)) = tokens.get(i + 1) {
                if g.delimiter() == Delimiter::Brace {
                    
                    let starts_with_upper = name.chars().next().unwrap_or('a').is_uppercase();

                    // 1. Skip if it's a field type (e.g. title: CustomTitle { ... })
                    let is_field_type = i > 0 && matches!(
                        tokens.get(i - 1),
                        Some(TokenTree::Punct(p)) if p.as_char() == ':'
                    );

                    // 2. Skip if it's preceded by 'if' (e.g. if is_maximized { ... })
                    let is_conditional = i > 0 && matches!(
                        tokens.get(i - 1),
                        Some(TokenTree::Ident(prev_id)) if prev_id.to_string() == "if"
                    );

                    // 3. Skip if preceded by '=' (e.g. let rect = Rect { ... })
                    let is_assignment = i > 0 && matches!(
                        tokens.get(i - 1),
                        Some(TokenTree::Punct(p)) if p.as_char() == '='
                    );

                    if starts_with_upper 
                        && !is_field_type 
                        && !is_conditional 
                        && !is_assignment
                        && !keywords.contains(&name.as_str()) 
                        && !zones.contains(&name) 
                    {
                        zones.push(name.clone());
                    }
                }
            }
        }
        
        // Recurse into ANY group
        if let TokenTree::Group(g) = &tokens[i] {
            scan_stream_for_zones(g.stream(), zones);
        }
        
        i += 1;
    }
}