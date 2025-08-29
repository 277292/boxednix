// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use sha_crypt::{sha512_simple, Sha512Params};

use super::{nix::Key, PatternEntry, QuotedString, Range, Replace, Result};

pub struct Sha512 {
    hash: QuotedString,
    range: Range,
}

impl Sha512 {
    fn hash(key: &str) -> Result<String> {
        let params = Sha512Params::default();
        sha512_simple(key, &params).map_err(|e| anyhow!("{:?}", e))
    }
}

impl Key for Sha512 {
    fn new(key: String, range: Range) -> Result<Option<Self>> {
        let hash = QuotedString::from(Self::hash(&key)?);

        Ok(Some(Self { hash, range }))
    }

    fn condition(text: &str) -> bool {
        text == PatternEntry::Sha512.as_ref()
    }
}

impl Replace for Sha512 {
    fn range(&self) -> Range {
        self.range.clone()
    }

    fn content(&self) -> &[u8] {
        self.hash.as_bytes()
    }
}
