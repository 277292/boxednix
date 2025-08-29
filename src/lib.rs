// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod editor;
mod file_io;
mod gitignore;
mod redact;
mod session;

use std::env;
use std::path::PathBuf;

use editor::EditorContext;
use redact::PatternEntry;
use session::{Session, SessionFile};

pub use anyhow::Result;
pub use log::{debug, error, info};

pub fn create_config(
    identity: PathBuf,
    passphrase: bool,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
) -> Result<()> {
    let cwd = env::current_dir()?;
    let config = config::create_default(&cwd, identity, recipients, recipients_files)?;
    if !config.identity.exists() {
        if let Some(parent) = config.identity.parent() {
            std::fs::create_dir_all(parent)?;
        }

        file_io::create_identity(&config.identity, passphrase)?;
    }
    Ok(())
}

pub fn run(source: PathBuf, editor: &str, editor_args: &[String]) -> Result<()> {
    let cwd = env::current_dir()?;
    let config = config::load(&cwd)?;
    let mut session_file = SessionFile::new(
        source,
        PatternEntry::to_module(),
        config.generate.dir,
        config.generate.gitignore,
        config.identity,
        config.recipients,
        config.recipients_files,
    )?;

    if session_file.source_exists() {
        file_io::decrypt(&mut session_file)?;
    } else {
        file_io::write(&mut session_file)?;
    }

    let editor_input = session_file.input();
    let session = Session::start(session_file, move |ctx: &mut SessionFile| {
        let prev_hash = ctx.hash();
        file_io::read(ctx)?;

        if prev_hash == ctx.hash() {
            return Ok(());
        }

        file_io::encrypt(ctx)?;
        info!("Encrypted changes back to source");

        match generate(ctx) {
            Ok(_) => info!("Generated file"),
            Err(e) => error!("File generation stopped because: {:?}", e),
        }

        Ok(())
    })?;

    editor::run(editor, editor_input, editor_args)?;
    session.stop()
}

fn generate(ctx: &mut SessionFile) -> Result<()> {
    if let Some(gitignore) = ctx.gitignore() {
        file_io::generate(gitignore)?;
    }
    file_io::generate(ctx)
}
