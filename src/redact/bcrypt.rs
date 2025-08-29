// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;

use super::{
    model::QuotedString,
    nix::Key,
    replace::{Range, Replace},
    PatternEntry, Result,
};

pub struct Bcrypt {
    hash: QuotedString,
    range: Range,
}

impl Bcrypt {
    fn hash(key: &str) -> Result<String> {
        bcrypt::hash(key, bcrypt::DEFAULT_COST).map_err(|e| anyhow!("{:?}", e))
    }
}

impl Key for Bcrypt {
    fn new(key: String, range: Range) -> Result<Option<Self>> {
        let hash = QuotedString::from(Self::hash(&key)?);

        Ok(Some(Self { hash, range }))
    }

    fn condition(text: &str) -> bool {
        text == PatternEntry::Bcrypt.as_ref()
    }
}

impl Replace for Bcrypt {
    fn range(&self) -> Range {
        self.range.clone()
    }

    fn content(&self) -> &[u8] {
        self.hash.as_bytes()
    }
}
