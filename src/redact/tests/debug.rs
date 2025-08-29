// SPDX-License-Identifier: GPL-3.0-only

use rnix::{NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken};

pub fn pretty_print_root(root: &SyntaxNode, highlights: &[&str]) {
    pretty_node(root, 0, highlights);
}

fn pretty_node(node: &SyntaxNode, indent: usize, highlights: &[&str]) {
    let indent_str = "  ".repeat(indent);
    let kind = format!("{:?}", node.kind());

    if let Some(inline) = try_inline_node(node, highlights) {
        println!("{}{}", indent_str, inline);
        return;
    }

    println!("{}{}[", indent_str, kind);
    for child in node.children_with_tokens() {
        match child {
            NodeOrToken::Node(n) => {
                pretty_node(&n, indent + 1, highlights);
            }
            NodeOrToken::Token(t) => {
                print_token(&t, indent + 1, highlights);
            }
        }
    }
    println!("{}]", indent_str);
}

fn try_inline_node(node: &SyntaxNode, highlights: &[&str]) -> Option<String> {
    let mut current = node.clone();
    let mut parts = Vec::new();

    loop {
        let mut iter = current.children_with_tokens();
        let first = iter.next()?;
        if iter.next().is_some() {
            return None; // Mehr als ein Kind → nicht inline
        }

        match first {
            NodeOrToken::Node(n) => {
                parts.push(format!("{:?}", current.kind()));
                current = n;
            }
            NodeOrToken::Token(t) => {
                let mut token_str = format!("{:?}", t.kind());

                // Nur annotieren, wenn direkt in NODE_IDENT enthalten
                if current.kind() == SyntaxKind::NODE_IDENT {
                    if let Some(label) = highlight_label(t.text(), highlights) {
                        token_str.push_str(&format!(" -> {:?}", label));
                    }
                }

                parts.push(format!("{:?}", current.kind()));
                parts.push(token_str);
                break;
            }
        }
    }

    // Rückgabe: korrekt geschlossene eckige Klammern
    let mut result = parts[0].clone();
    for part in parts.iter().skip(1) {
        result = format!("{}[{}", result, part);
    }
    Some(format!("{}{}", result, "]".repeat(parts.len() - 1)))
}

fn print_token(token: &SyntaxToken, indent: usize, highlights: &[&str]) {
    let indent_str = "  ".repeat(indent);
    let mut out = format!("{:?}", token.kind());

    if let Some(label) = highlight_label(token.text(), highlights) {
        out.push_str(&format!(" -> {:?}", label));
    }

    println!("{}{}", indent_str, out);
}

fn highlight_label<'a>(text: &'a str, highlights: &'a [&'a str]) -> Option<&'a str> {
    let trimmed = text.trim_matches('"');
    highlights.iter().copied().find(|&h| h == trimmed)
}
