// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use hex::encode;
use pbkdf2::{hmac::Hmac, pbkdf2};
use sha1::Sha1;

use super::{
    model::QuotedString,
    nix::SaltAndKey,
    replace::{Range, Replace},
    PatternEntry, Result,
};

pub struct Psk {
    derivation: QuotedString,
    range: Range,
}

impl Psk {
    fn derive(ssid: &str, key: &str) -> Result<String> {
        let mut psk = [0u8; 32];
        pbkdf2::<Hmac<Sha1>>(key.as_bytes(), ssid.as_bytes(), 4096, &mut psk)
            .map_err(|e| anyhow!("{:?}", e))?;
        Ok(encode(psk))
    }
}

impl SaltAndKey for Psk {
    fn new(salt: String, key: String, range: Range) -> Result<Option<Self>> {
        let derivation = QuotedString::from(Self::derive(&salt, &key)?);

        Ok(Some(Self { derivation, range }))
    }

    fn condition(text: &str) -> bool {
        text == PatternEntry::Psk.as_ref()
    }
}

impl Replace for Psk {
    fn range(&self) -> Range {
        self.range.clone()
    }

    fn content(&self) -> &[u8] {
        self.derivation.as_bytes()
    }
}
