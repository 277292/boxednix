// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

pub struct QuotedString(String);

impl From<&str> for QuotedString {
    fn from(value: &str) -> Self {
        Self(quoted(value))
    }
}

impl From<String> for QuotedString {
    fn from(value: String) -> Self {
        Self(quoted(&value))
    }
}

impl Deref for QuotedString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn quoted(value: &str) -> String {
    let mut buf = String::with_capacity(value.len() + 2);
    buf.push('"');
    buf.push_str(value);
    buf.push('"');

    buf
}
