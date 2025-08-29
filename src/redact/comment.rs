// SPDX-License-Identifier: GPL-3.0-only

use super::{nix::Position, Range, Replace};

pub struct Comment {
    range: Range,
}

impl Position for Comment {
    fn new(range: Range) -> Option<Self> {
        Some(Self { range })
    }

    fn condition(_: &str) -> bool {
        true
    }
}

impl Replace for Comment {
    fn range(&self) -> Range {
        self.range.clone()
    }

    fn content(&self) -> &[u8] {
        &[]
    }
}
