// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;

use crate::Result;

pub trait EditorContext {
    fn input(&self) -> PathBuf;
}

pub fn run(editor: &str, input: PathBuf, args: &[String]) -> Result<()> {
    Command::new(editor)
        .arg(input)
        .args(args)
        .status()
        .context(format!("Failed to launch '{}'", editor))?;
    Ok(())
}
