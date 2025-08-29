// SPDX-License-Identifier: GPL-3.0-only

use indoc::indoc;
use strum::VariantNames;
use strum_macros::{AsRefStr, EnumIter, EnumString, VariantNames};

use super::{nix::Position, Range, Replace};

#[derive(EnumIter, EnumString, VariantNames, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum PatternEntry {
    Bcrypt,
    Sha512,
    Psk,
}

impl PatternEntry {
    pub fn to_module() -> Vec<u8> {
        format!(
            indoc! {r#"
                {{
                  {}
                }}: {{
                  
                }}
            "#},
            Self::VARIANTS.join(",\n  ")
        )
        .into_bytes()
    }
}

pub struct Pattern {
    range: Range,
}

impl Position for Pattern {
    fn new(range: Range) -> Option<Self> {
        Some(Self { range })
    }

    fn condition(text: &str) -> bool {
        PatternEntry::VARIANTS.iter().any(|&entry| entry == text)
    }
}

impl Replace for Pattern {
    fn range(&self) -> Range {
        self.range.clone()
    }

    fn content(&self) -> &[u8] {
        &[]
    }
}
