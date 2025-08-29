// SPDX-License-Identifier: GPL-3.0-only

mod bcrypt;
mod comment;
mod model;
mod nix;
mod pattern;
mod psk;
mod replace;
mod sha;

#[cfg(test)]
mod tests;

use std::{cmp::Reverse, collections::HashSet};

use crate::Result;

use anyhow::anyhow;
use bcrypt::Bcrypt;
use comment::Comment;
use model::QuotedString;
use nix::NodeOrToken;
use pattern::Pattern;
use psk::Psk;
use replace::{Range, Replace};
use sha::Sha512;

pub(crate) use pattern::PatternEntry;

pub fn process(content: &[u8]) -> Result<Vec<u8>> {
    let mut pattern = HashSet::new();
    let mut replacments: Vec<Box<dyn Replace>> = Vec::new();

    for node_or_token in nix::parse(std::str::from_utf8(content)?) {
        match node_or_token {
            NodeOrToken::Node(node) => {
                if nix::is_node_error(&node) {
                    return Err(anyhow!("Syntax Error!"));
                }
                if let Some(pat) = nix::parse_pattern(&node) {
                    pattern.insert(pat);
                    continue;
                }
                if let Some(bcrypt) = nix::parse_key_for::<Bcrypt>(&node)? {
                    replacments.push(Box::new(bcrypt));
                    continue;
                }
                if let Some(sha512) = nix::parse_key_for::<Sha512>(&node)? {
                    replacments.push(Box::new(sha512));
                    continue;
                }
                if let Some(psk) = nix::parse_salt_and_key_for::<Psk>(&node)? {
                    replacments.push(Box::new(psk));
                    continue;
                }
            }
            NodeOrToken::Token(token) => {
                if nix::is_token_error(&token) {
                    return Err(anyhow!("Syntax Error!"));
                }
                if let Some(comment) = nix::parse_comment_for::<Comment>(&token) {
                    replacments.push(Box::new(comment));
                    continue;
                }
            }
        }
    }

    replacments.extend(
        pattern
            .iter()
            .filter_map(nix::resolve_pattern_for::<Pattern>)
            .flat_map(|pattern| pattern.into_iter())
            .map(|pattern| Box::new(pattern) as Box<_>),
    );

    replacments.sort_by_key(|replace| Reverse(replace.range().start));

    let mut output = content.to_vec();
    for replace in replacments {
        output.splice(replace.range(), replace.content().iter().copied());
    }

    Ok(output)
}
