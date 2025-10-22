// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod editor;
mod file_io;
mod redact;
mod session;

use std::path::PathBuf;
use std::{env, process::Command};

use editor::EditorContext;
use redact::PatternEntry;
use session::{Session, SessionFile};

pub use anyhow::{Context, Result};
pub use log::{debug, error, info};

pub fn create_config(
    identity: PathBuf,
    dir: Option<PathBuf>,
    passphrase: bool,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
) -> Result<()> {
    let create_flake = matches!(&dir, Some(dir) if dir.is_absolute()) || dir.is_none();

    let cwd = env::current_dir()?;
    let config = config::create_default(&cwd, identity, dir, recipients, recipients_files)?;
    if !config.identity.exists() {
        if let Some(parent) = config.identity.parent() {
            std::fs::create_dir_all(parent)?;
        }

        file_io::create_identity(&config.identity, passphrase)?;
    }

    if create_flake {
        if let Some(parent) = config.generated_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(
            config.generated_dir.join("flake.nix"),
            include_str!("../assets/default_flake.nix"),
        )?;
    }

    Ok(())
}

pub fn run(source: PathBuf, editor: &str, editor_args: &[String]) -> Result<()> {
    let cwd = env::current_dir()?;
    let config = config::load(&cwd, &source)?;
    let mut session_file = SessionFile::new(
        source,
        PatternEntry::to_module(),
        config.file_name,
        config.target_dir,
        config.identity,
        config.recipients,
        config.recipients_files,
        config.flake_input,
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

        match file_io::generate(ctx) {
            Ok(_) => {
                info!("Generated file");
                update_lock_file(ctx.flake_input())?;
            }
            Err(e) => error!("File generation stopped because: {:?}", e),
        }

        Ok(())
    })?;

    editor::run(editor, editor_input, editor_args)?;
    session.stop()
}

fn update_lock_file(flake_input: Option<String>) -> Result<()> {
    if let Some(input) = flake_input {
        Command::new("nix")
            .arg("flake")
            .arg("update")
            .arg(&input)
            // .status()
            .output()
            .context(format!("Failed to update flake input '{}'", input))?;
    }

    Ok(())
}
