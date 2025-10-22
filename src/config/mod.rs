// SPDX-License-Identifier: GPL-3.0-only

mod model;

use anyhow::anyhow;
use directories_next::ProjectDirs;
use model::TomlConfig;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::Result;

pub use model::Config;

const CONFIG_FILE: &str = "boxednix.toml";

pub fn create_default(
    cwd: &Path,
    identity: PathBuf,
    dir: Option<PathBuf>,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
) -> Result<TomlConfig> {
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

    let generated_dir = match dir {
        Some(dir) => dir,
        None => generated_dir(cwd)?,
    };

    let config = TomlConfig {
        identity,
        recipients,
        recipients_files,
        generated_dir,
        update_flake_input: None,
    };

    let toml = toml::to_string_pretty(&config)?;
    std::fs::write(output, toml)?;
    Ok(config)
}

pub fn load(cwd: &Path, source: &Path) -> Result<Config> {
    let source_dir = source
        .parent()
        .ok_or(anyhow!("can't get parent of source ({:?}).)", source))?;

    let project_root = find_project_root(&cwd).ok_or(anyhow!(
        "Config file not found. Current working dir: {:?}",
        cwd
    ))?;

    let config_path = project_root.join(CONFIG_FILE);
    let toml = std::fs::read_to_string(config_path)?;
    let toml_config: TomlConfig = toml::from_str(&toml)?;

    Ok(Config {
        identity: toml_config.identity,
        recipients: toml_config.recipients,
        recipients_files: toml_config.recipients_files,
        file_name: file_name(&source)?,
        target_dir: target_dir(&cwd, &project_root, &source_dir, &toml_config.generated_dir)?,
        flake_input: toml_config.update_flake_input,
    })
}

fn target_dir(cwd: &Path, root: &Path, source_dir: &Path, target_dir: &Path) -> Result<PathBuf> {
    if target_dir.is_absolute() {
        cwd.join(source_dir)
            .strip_prefix(root)
            .map(|sub_dir| target_dir.join(sub_dir))
            .map_err(|e| {
                anyhow!(
                    "failed to strip prefix {:?} from {:?}. error: {}",
                    root,
                    cwd,
                    e
                )
            })
    } else {
        Ok(source_dir.join(target_dir))
    }
}

fn file_name(source: &Path) -> Result<OsString> {
    let mut stem = source
        .file_stem()
        .map(OsString::from)
        .ok_or(anyhow!("source has no stem, {:?}", source))?;

    stem.push(".nix");
    Ok(stem)
}

pub fn find_project_root<'a>(cwd: &'a Path) -> Option<&'a Path> {
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
        user_dir().map(|root| root.join(&input))
    } else {
        Ok(input)
    }
}

fn user_dir<'a>() -> Result<PathBuf> {
    ProjectDirs::from("tlm", "depeh", "boxednix")
        .map(|project| project.config_dir().to_path_buf())
        .ok_or(anyhow!("can't find user dir"))
}

fn generated_dir(cwd: &Path) -> Result<PathBuf> {
    let user_dir = user_dir()?;
    cwd.file_name()
        .map(|name| user_dir.join("generated").join(name))
        .ok_or(anyhow!("current working dir has no name: {:?}", cwd))
}
