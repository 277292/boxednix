// SPDX-License-Identifier: GPL-3.0-only

use rnix::{
    Root,
    SyntaxKind::{
        self, NODE_APPLY, NODE_ERROR, NODE_IDENT, NODE_PAREN, NODE_PATTERN, NODE_PAT_ENTRY,
        NODE_STRING, TOKEN_COLON, TOKEN_COMMENT, TOKEN_ELLIPSIS, TOKEN_ERROR, TOKEN_R_BRACE,
        TOKEN_STRING_CONTENT, TOKEN_WHITESPACE,
    },
    SyntaxNode, SyntaxToken, TextRange, TextSize,
};

use super::{Range, Result};

pub use rnix::NodeOrToken;

pub fn parse(nix: &str) -> impl Iterator<Item = NodeOrToken<SyntaxNode, SyntaxToken>> {
    Root::parse(nix)
        .syntax()
        .descendants_with_tokens()
        .filter(|node_or_token| {
            matches!(
                node_or_token.kind(),
                NODE_IDENT | TOKEN_COMMENT | NODE_ERROR | TOKEN_ERROR
            )
        })
}

pub fn parse_key_for<K: Key>(node: &SyntaxNode) -> Result<Option<K>> {
    if !K::condition(&node.text().to_string()) {
        return Ok(None);
    }

    let mut key = None;
    let mut range = None;

    for (parent, kinds) in buffered_ancestors(node, 2) {
        match kinds.as_slice() {
            [NODE_APPLY] => {
                key = parse_key_from_apply(&parent);
                range = Some(parent.text_range());
            }
            [NODE_APPLY, NODE_PAREN] => {
                range = Some(parent.text_range());
            }
            _ => break,
        };
    }

    match (key, range) {
        (Some(key), Some(range)) => K::new(key, range.to_range()),
        _ => Ok(None),
    }
}

pub fn parse_salt_and_key_for<SK: SaltAndKey>(node: &SyntaxNode) -> Result<Option<SK>> {
    if !SK::condition(&node.text().to_string()) {
        return Ok(None);
    }

    let mut salt = None;
    let mut key = None;
    let mut range = None;

    for (parent, kinds) in buffered_ancestors(node, 3) {
        match kinds.as_slice() {
            [NODE_APPLY] => {
                salt = parse_key_from_apply(&parent);
            }
            [NODE_APPLY, NODE_APPLY] => {
                key = parse_key_from_apply(&parent);
                range = Some(parent.text_range());
            }
            [NODE_APPLY, NODE_APPLY, NODE_PAREN] => {
                range = Some(parent.text_range());
            }
            _ => break,
        };
    }

    match (salt, key, range) {
        (Some(salt), Some(key), Some(range)) => SK::new(salt, key, range.to_range()),
        _ => Ok(None),
    }
}

pub fn parse_comment_for<P: Position>(token: &SyntaxToken) -> Option<P> {
    if !(token.kind() == TOKEN_COMMENT && P::condition(token.text())) {
        return None;
    }

    let start = token
        .siblings_with_tokens(rowan::Direction::Prev)
        .skip(1)
        .find(|prev| prev.kind() != TOKEN_WHITESPACE)
        .map(|prev| prev.text_range().end())
        .unwrap_or(token.text_range().start());
    let end = token.text_range().end();

    P::new(start.to_usize()..end.to_usize())
}

pub fn parse_pattern(node: &SyntaxNode) -> Option<SyntaxNode> {
    let entry = node
        .parent()
        .take_if(|parent| parent.kind() == NODE_PAT_ENTRY)?;

    entry
        .parent()
        .take_if(|parent| parent.kind() == NODE_PATTERN)
}

pub fn resolve_pattern_for<P: Position>(node: &SyntaxNode) -> Option<Vec<P>> {
    let children_count = node.children().count();
    let pattern_entry: Vec<_> = node
        .children()
        .filter(|child| P::condition(&child.text().to_string()))
        .collect();

    if children_count == pattern_entry.len() {
        let start = node.text_range().start();
        let end = node
            .siblings_with_tokens(rowan::Direction::Next)
            .skip(1)
            .find(|sibling| !matches!(sibling.kind(), TOKEN_WHITESPACE | TOKEN_COLON))
            .map(|sibling| sibling.text_range().start())
            .unwrap_or(node.text_range().end());

        let pattern = P::new(start.to_usize()..end.to_usize())?;
        return Some(vec![pattern]);
    }

    pattern_entry
        .iter()
        .map(|entry| {
            let start = entry.text_range().start();
            let end = entry
                .siblings_with_tokens(rowan::Direction::Next)
                .skip(1)
                .find(|sibling| {
                    matches!(
                        sibling.kind(),
                        NODE_PAT_ENTRY | TOKEN_ELLIPSIS | TOKEN_R_BRACE
                    )
                })
                .map(|sibling| sibling.text_range().start())
                .unwrap_or(entry.text_range().end());

            P::new(start.to_usize()..end.to_usize())
        })
        .collect()
}

pub fn is_node_error(node: &SyntaxNode) -> bool {
    node.kind() == NODE_ERROR
}

pub fn is_token_error(token: &SyntaxToken) -> bool {
    token.kind() == TOKEN_ERROR
}

fn parse_key_from_apply(node: &SyntaxNode) -> Option<String> {
    let node = node
        .children()
        .find(|children| children.kind() == NODE_STRING)?;

    node.children_with_tokens().find_map(|child_or_token| {
        child_or_token
            .as_token()
            .filter(|token| token.kind() == TOKEN_STRING_CONTENT)
            .map(|token| token.text().to_owned())
    })
}

fn buffered_ancestors(
    node: &SyntaxNode,
    buf_size: usize,
) -> impl Iterator<Item = (SyntaxNode, Vec<SyntaxKind>)> {
    let mut iter = node.ancestors().skip(1);
    let mut kinds: Vec<SyntaxKind> = Vec::with_capacity(buf_size);

    std::iter::from_fn(move || {
        let current = iter.next()?;
        kinds.push(current.kind());

        if kinds.len() > buf_size {
            return None;
        }

        Some((current, kinds.clone()))
    })
}

pub trait Key: Sized {
    fn new(key: String, range: Range) -> Result<Option<Self>>;
    fn condition(text: &str) -> bool;
}

pub trait SaltAndKey: Sized {
    fn new(salt: String, key: String, range: Range) -> Result<Option<Self>>;
    fn condition(text: &str) -> bool;
}

pub trait Position: Sized {
    fn new(range: Range) -> Option<Self>;
    fn condition(text: &str) -> bool;
}

trait TextRangeExt {
    fn to_range(&self) -> Range;
}

impl TextRangeExt for TextRange {
    fn to_range(&self) -> Range {
        usize::from(self.start())..usize::from(self.end())
    }
}

trait TextSizeExt {
    fn to_usize(&self) -> usize;
}

impl TextSizeExt for TextSize {
    fn to_usize(&self) -> usize {
        usize::from(*self)
    }
}
