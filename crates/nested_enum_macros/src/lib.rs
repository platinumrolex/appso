use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use core::str::FromStr;

// ---------------------------------------------------------------------
//  r#enum!
// ---------------------------------------------------------------------
#[proc_macro]
pub fn r#enum(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut pos = 0;

    let outer_attrs = collect_attributes(&tokens, &mut pos);

    let vis = if pos < tokens.len() && is_ident(&tokens[pos], "pub") {
        pos += 1;
        "pub"
    } else {
        ""
    };

    if pos < tokens.len() && is_ident(&tokens[pos], "enum") {
        pos += 1;
    }

    let name = expect_ident(&tokens, &mut pos, "enum name");

    let body = match tokens.get(pos) {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
            pos += 1;
            g.clone()
        }
        other => panic!("Expected Brace group, got {:?}", other),
    };

    let mut inner_defs = Vec::new();
    let mut leaves: Vec<(Ident, String)> = Vec::new();
    
    // Pass a chain of idents so we can correctly generate tuple-wrapping patterns
    parse_variants_inner(
        body.stream().into_iter().collect(),
        vec![name.clone()],
        vis,
        &mut inner_defs,
        &mut leaves,
    );

    let mut out = TokenStream::new();
    
    // 1) Outer enum
    out.extend(outer_attrs.into_iter());
    if vis == "pub" {
        out.extend(std::iter::once(TokenTree::Ident(Ident::new("pub", Span::call_site()))));
    }
    out.extend(std::iter::once(TokenTree::Ident(Ident::new("enum", Span::call_site()))));
    out.extend(std::iter::once(TokenTree::Ident(name.clone())));
    out.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Brace, build_outer_variants(body.stream())))));

    // 2) Inner enum definitions
    for inner in &inner_defs {
        out.extend(inner.clone().into_iter());
    }

    // 3) Match macro
    out.extend(build_match_macro(&name, &leaves).into_iter());

    out
}

// ---------------------------------------------------------------------
//  match_nested!
// ---------------------------------------------------------------------
#[proc_macro]
pub fn match_nested(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    if tokens.is_empty() { return TokenStream::new(); }
    
    let ty = match &tokens[0] {
        TokenTree::Ident(id) => id.clone(),
        _ => panic!("Expected type name"),
    };
    
    let comma_idx = tokens.iter()
        .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ','))
        .expect("Expected comma after type in match_nested");
    
    let rest_stream: TokenStream = tokens[comma_idx + 1..].iter().cloned().collect();
    
    let mut out = TokenStream::new();
    out.extend(TokenStream::from_str(&format!("match_{}!", ty)).unwrap());
    
    let group = TokenTree::Group(Group::new(Delimiter::Parenthesis, rest_stream));
    out.extend(std::iter::once(group));
    
    out
}

// =====================================================================
//              Helper Functions
// =====================================================================

fn expect_ident(tokens: &[TokenTree], pos: &mut usize, desc: &str) -> Ident {
    match tokens.get(*pos) {
        Some(TokenTree::Ident(id)) => {
            *pos += 1;
            id.clone()
        }
        other => panic!("Expected {} at position {}, got {:?}", desc, *pos, other),
    }
}

fn is_ident(tt: &TokenTree, s: &str) -> bool {
    matches!(tt, TokenTree::Ident(id) if id.to_string() == s)
}

fn collect_attributes(tokens: &[TokenTree], pos: &mut usize) -> Vec<TokenTree> {
    let mut attrs = Vec::new();
    while *pos < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[*pos] {
            if p.as_char() == '#' && *pos + 1 < tokens.len() {
                if let TokenTree::Group(_) = &tokens[*pos + 1] {
                    attrs.push(tokens[*pos].clone());
                    attrs.push(tokens[*pos + 1].clone());
                    *pos += 2;
                    continue;
                }
            }
        }
        break;
    }
    attrs
}

// Generates A::B(B::C(C::D)) string matching patterns for nested leaves.
fn build_nested_pattern(chain: &[Ident]) -> String {
    if chain.len() == 1 {
        return chain[0].to_string();
    }
    let mut pat = format!("{}::{}", chain[0], chain[1]);
    for i in 1..chain.len() - 1 {
        pat.push_str(&format!("({}::{}", chain[i], chain[i+1]));
    }
    for _ in 1..chain.len() - 1 {
        pat.push_str(")");
    }
    pat
}

// ---------------------------------------------------------------------
//   Recursive parsing
// ---------------------------------------------------------------------
fn parse_variants_inner(
    tokens: Vec<TokenTree>,
    chain: Vec<Ident>,
    vis: &str,
    inner_defs: &mut Vec<TokenStream>,
    leaves: &mut Vec<(Ident, String)>,
) {
    let mut pos = 0;
    let len = tokens.len();
    while pos < len {
        let attrs = collect_attributes(&tokens, &mut pos);
        if pos >= len { break; }
        let variant = expect_ident(&tokens, &mut pos, "variant name");

        let mut has_children = false;
        if pos < len {
            if let TokenTree::Group(g) = &tokens[pos] {
                if g.delimiter() == Delimiter::Brace {
                    has_children = true;
                    pos += 1;
                    let children_stream = g.stream();

                    let inner = build_inner_enum(vis, &variant, children_stream.clone(), &attrs);
                    inner_defs.push(inner);

                    let mut new_chain = chain.clone();
                    new_chain.push(variant.clone());
                    parse_variants_inner(
                        children_stream.into_iter().collect(),
                        new_chain,
                        vis,
                        inner_defs,
                        leaves,
                    );
                }
            }
        }

        if !has_children {
            let mut leaf_chain = chain.clone();
            leaf_chain.push(variant.clone());
            leaves.push((variant, build_nested_pattern(&leaf_chain)));
        }

        if pos < len {
            if let TokenTree::Punct(p) = &tokens[pos] {
                if p.as_char() == ',' {
                    pos += 1;
                }
            }
        }
    }
}

// Emits A(A) properly using Groups, bypassing the parenthesis Punct panic.
fn build_outer_variants(body: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = body.into_iter().collect();
    let mut out = TokenStream::new();
    let mut pos = 0;
    while pos < tokens.len() {
        let attrs = collect_attributes(&tokens, &mut pos);
        out.extend(attrs.into_iter());
        if pos >= tokens.len() { break; }
        let ident = expect_ident(&tokens, &mut pos, "variant");
        
        let has_children = pos < tokens.len()
            && matches!(&tokens[pos], TokenTree::Group(g) if g.delimiter() == Delimiter::Brace);

        out.extend(std::iter::once(TokenTree::Ident(ident.clone())));
        
        if has_children {
            let inner = TokenTree::Ident(ident);
            let group = TokenTree::Group(Group::new(Delimiter::Parenthesis, std::iter::once(inner).collect()));
            out.extend(std::iter::once(group));
            pos += 1; // skip brace group
        }
        
        if pos < tokens.len() && matches!(&tokens[pos], TokenTree::Punct(p) if p.as_char() == ',') {
            out.extend(std::iter::once(tokens[pos].clone()));
            pos += 1;
        }
    }
    out
}

fn build_inner_enum(vis: &str, name: &Ident, body: TokenStream, attrs: &[TokenTree]) -> TokenStream {
    let mut ts = TokenStream::new();
    
    ts.extend(TokenStream::from_str("#[derive(Clone, Copy, PartialEq, Eq, Debug)]").unwrap());
    ts.extend(attrs.iter().cloned());
    
    if vis == "pub" {
        ts.extend(TokenStream::from_str("pub ").unwrap());
    }
    
    ts.extend(std::iter::once(TokenTree::Ident(Ident::new("enum", Span::call_site()))));
    ts.extend(std::iter::once(TokenTree::Ident(name.clone())));
    
    // THE FIX IS HERE: We process the body through build_outer_variants 
    // so inner enums get `Variant(Variant)` instead of struct-like fields.
    let processed_body = build_outer_variants(body);
    let body_group = TokenTree::Group(Group::new(Delimiter::Brace, processed_body));
    ts.extend(std::iter::once(body_group));
    
    ts
}

// Uses String manipulation to bypass complicated multi-token spacing issues entirely.
// Creates a recursive TT Muncher to map arbitrary matching into valid Rust nested enums.
fn build_match_macro(name: &Ident, leaves: &[(Ident, String)]) -> TokenStream {
    let macro_name = format!("match_{}", name);
    let mut source = String::new();

    source.push_str(&format!("macro_rules! {} {{\n", macro_name));
    
    // Base Case
    source.push_str("    (@build ($action:expr) ($($arms:tt)*) ()) => {\n");
    source.push_str("        match $action { $($arms)* }\n");
    source.push_str("    };\n");

    for (leaf, path_str) in leaves {
        // 1. Handled standard comma separator
        source.push_str(&format!("    (@build ($action:expr) ($($arms:tt)*) ({} => $body:expr, $($rest:tt)*)) => {{\n", leaf));
        source.push_str(&format!("        {}!(@build ($action) ($($arms)* {} => $body,) ($($rest)*))\n", macro_name, path_str));
        source.push_str("    };\n");

        // 2. Handled blocks without commas (standard Rust behavior)
        source.push_str(&format!("    (@build ($action:expr) ($($arms:tt)*) ({} => $body:block $($rest:tt)*)) => {{\n", leaf));
        source.push_str(&format!("        {}!(@build ($action) ($($arms)* {} => $body,) ($($rest)*))\n", macro_name, path_str));
        source.push_str("    };\n");

        // 3. Handled end of list items
        source.push_str(&format!("    (@build ($action:expr) ($($arms:tt)*) ({} => $body:expr)) => {{\n", leaf));
        source.push_str(&format!("        {}!(@build ($action) ($($arms)* {} => $body,) ())\n", macro_name, path_str));
        source.push_str("    };\n");
    }

    // Wildcards
    source.push_str("    (@build ($action:expr) ($($arms:tt)*) (_ => $body:expr, $($rest:tt)*)) => {\n");
    source.push_str(&format!("        {}!(@build ($action) ($($arms)* _ => $body,) ($($rest)*))\n", macro_name));
    source.push_str("    };\n");
    source.push_str("    (@build ($action:expr) ($($arms:tt)*) (_ => $body:block $($rest:tt)*)) => {\n");
    source.push_str(&format!("        {}!(@build ($action) ($($arms)* _ => $body,) ($($rest)*))\n", macro_name));
    source.push_str("    };\n");
    source.push_str("    (@build ($action:expr) ($($arms:tt)*) (_ => $body:expr)) => {\n");
    source.push_str(&format!("        {}!(@build ($action) ($($arms)* _ => $body,) ())\n", macro_name));
    source.push_str("    };\n");

    // Entry Point Matcher
    source.push_str("    ($action:expr, { $($arms:tt)* }) => {\n");
    source.push_str(&format!("        {}!(@build ($action) () ($($arms)*))\n", macro_name));
    source.push_str("    };\n");

    source.push_str("}\n");

    TokenStream::from_str(&source).expect("Failed to parse internally generated match macro string")
}