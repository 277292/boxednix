// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use crate::{file_io::GenerationContext, Result};

pub struct Gitignore {
    target: PathBuf,
    content: String,
}

impl Gitignore {
    pub fn new(file: PathBuf) -> Self {
        Self {
            target: file,
            content: String::from("*\n!.gitifnore\n"),
        }
    }
}

impl GenerationContext for Gitignore {
    fn input(&self) -> Result<Vec<u8>> {
        Ok(self.content.clone().into_bytes())
    }

    fn output(&self) -> Option<String> {
        self.target.to_str().map(String::from)
    }
}
