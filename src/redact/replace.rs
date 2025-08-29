// SPDX-License-Identifier: GPL-3.0-only

pub type Range = std::ops::Range<usize>;

pub trait Replace {
    fn range(&self) -> Range;
    fn content(&self) -> &[u8];
}
