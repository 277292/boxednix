// SPDX-License-Identifier: GPL-3.0-only

mod model;

use anyhow::anyhow;
use directories_next::ProjectDirs;
use std::path::{Path, PathBuf};

use crate::Result;

pub use model::{Config, Generate, Gitignore};

const CONFIG_FILE: &str = "boxednix.toml";
const GENERATE_DIR: &str = ".boxednix";

pub fn create_default(
    cwd: &Path,
    identity: PathBuf,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
) -> Result<Config> {
    let output = cwd.join(CONFIG_FILE);
    if output.exists() {
        return Err(anyhow!("A configuration already exists"));
    }

    let identity = resolve_path(identity)?;
    let recipients = recipients
        .into_iter()
        .map(resolve_path)
        .collect::<Result<_, _>>()?;

    let recipients_files = recipients_files
        .into_iter()
        .map(resolve_path)
        .collect::<Result<_, _>>()?;

    let config = Config {
        identity,
        recipients,
        recipients_files,
        generate: Generate {
            dir: PathBuf::from(GENERATE_DIR),
            gitignore: Gitignore::Always,
        },
    };

    let toml = toml::to_string_pretty(&config)?;
    std::fs::write(output, toml)?;
    Ok(config)
}

pub fn load(cwd: &Path) -> Result<Config> {
    let project_root = find_project_root(&cwd).ok_or(anyhow!("Config file not found"))?;
    let config_path = project_root.join(CONFIG_FILE);
    let contents = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}

fn find_project_root<'a>(cwd: &'a Path) -> Option<&'a Path> {
    let mut current = cwd;

    loop {
        let candidate = current.join(CONFIG_FILE);
        if candidate.exists() {
            return Some(current);
        }

        current = match current.parent() {
            Some(parent) => parent,
            None => return None,
        };
    }
}

fn resolve_path(input: PathBuf) -> Result<PathBuf> {
    if input.components().count() == 1 && !input.is_absolute() {
        Ok(default_root_dir()
            .map(|root| root.join(&input))
            .ok_or(anyhow!("invalid path: {:?}", input))?)
    } else {
        Ok(input)
    }
}

fn default_root_dir<'a>() -> Option<PathBuf> {
    ProjectDirs::from("tlm", "depeh", "boxednix").map(|project| project.config_dir().to_path_buf())
}
