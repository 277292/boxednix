// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use blake3::Hash;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

use super::session::WatcherContext;
use crate::{
    config,
    editor::EditorContext,
    file_io::{
        DecryptionContext, EncryptionContext, GenerationContext, ReadingContext, WritingContext,
    },
    gitignore::Gitignore,
    redact, Result,
};

pub struct SessionFile {
    path: PathBuf,
    dir: TempDir,
    hash: Option<Hash>,
    content: Vec<u8>,
    source: PathBuf,
    target: PathBuf,
    gitignore: Option<Gitignore>,
    identity: PathBuf,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
}

impl SessionFile {
    pub fn new(
        source: PathBuf,
        template: Vec<u8>,
        target_dir: PathBuf,
        gitignore: config::Gitignore,
        identity: PathBuf,
        recipients: Vec<PathBuf>,
        recipients_files: Vec<PathBuf>,
    ) -> Result<Self> {
        let name = Self::file_name(&source)?;
        let target = Self::create_target(&source, &target_dir, &name)?;
        let (path, dir) = Self::create_temp_file(&name)?;

        let gitignore = match gitignore {
            config::Gitignore::Always => Some(Gitignore::new(target_dir)),
            config::Gitignore::None => None,
        };

        Ok(Self {
            path,
            dir,
            hash: None,
            content: template,
            source,
            target,
            gitignore,
            identity,
            recipients,
            recipients_files,
        })
    }

    pub fn hash(&self) -> Option<Hash> {
        self.hash.to_owned()
    }

    pub fn source_exists(&self) -> bool {
        self.source.exists()
    }

    pub fn exists_in(&self, other: &[PathBuf]) -> bool {
        other.iter().any(|path| path == &self.path)
    }

    pub fn gitignore(&self) -> Option<&Gitignore> {
        self.gitignore.as_ref()
    }

    fn create_target(source: &Path, target_dir: &Path, file_name: &OsString) -> Result<PathBuf> {
        source
            .parent()
            .map(|root| root.join(target_dir).join(&file_name))
            .ok_or(anyhow!("source has no parent: {:?}", source))
    }

    fn create_temp_file(name: &OsString) -> Result<(PathBuf, TempDir)> {
        let dir = TempDir::new()?;
        let path = dir.path().join(&name);

        Ok((path, dir))
    }

    fn file_name(path: &Path) -> Result<OsString> {
        let mut stem = path
            .file_stem()
            .map(OsString::from)
            .ok_or(anyhow!("path has not stem, {:?}", path))?;

        stem.push(".nix");
        Ok(stem)
    }
}

impl ReadingContext for SessionFile {
    fn input(&self) -> Option<String> {
        self.path.to_str().map(String::from)
    }

    fn output(&mut self, content: Vec<u8>, hash: Hash) {
        self.content = content;
        self.hash = Some(hash);
    }
}

impl WritingContext for SessionFile {
    fn input(&self) -> &[u8] {
        &self.content
    }

    fn output(&self) -> Option<String> {
        self.path.to_str().map(String::from)
    }

    fn result(&mut self, hash: Hash) {
        self.hash = Some(hash);
    }
}

impl DecryptionContext for SessionFile {
    fn input(&self) -> Option<String> {
        self.source.to_str().map(String::from)
    }

    fn output(&self) -> Option<String> {
        self.path.to_str().map(String::from)
    }

    fn identities(&self) -> Vec<String> {
        match self.identity.to_str().map(String::from) {
            Some(identites) => vec![identites],
            None => vec![],
        }
    }

    fn result(&mut self, hash: Hash) {
        self.hash = Some(hash);
    }
}

impl EncryptionContext for SessionFile {
    fn input(&self) -> &[u8] {
        &self.content
    }

    fn output(&self) -> Option<String> {
        self.source.to_str().map(String::from)
    }

    fn identities(&self) -> Vec<String> {
        match self.identity.to_str().map(String::from) {
            Some(identites) => vec![identites],
            None => vec![],
        }
    }

    fn recipients(&self) -> Vec<String> {
        self.recipients
            .iter()
            .filter_map(|path| path.to_str().map(String::from))
            .collect()
    }

    fn recipients_files(&self) -> Vec<String> {
        self.recipients_files
            .iter()
            .filter_map(|path| path.to_str().map(String::from))
            .collect()
    }
}

impl GenerationContext for SessionFile {
    fn input(&self) -> Result<Vec<u8>> {
        redact::process(&self.content)
    }

    fn output(&self) -> Option<String> {
        self.target.to_str().map(String::from)
    }
}

impl EditorContext for SessionFile {
    fn input(&self) -> PathBuf {
        self.path.clone()
    }
}

impl WatcherContext for SessionFile {
    fn target(&self) -> &Path {
        &self.dir.path()
    }

    fn target_file(&self) -> &Path {
        &self.path
    }
}
