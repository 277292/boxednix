// SPDX-License-Identifier: GPL-3.0-only

use blake3::Hash;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

use super::session::WatcherContext;
use crate::{
    editor::EditorContext,
    file_io::{
        DecryptionContext, EncryptionContext, GenerationContext, ReadingContext, WritingContext,
    },
    redact, Result,
};

pub struct SessionFile {
    path: PathBuf,
    dir: TempDir,
    hash: Option<Hash>,
    content: Vec<u8>,
    source: PathBuf,
    target: PathBuf,
    identity: PathBuf,
    recipients: Vec<PathBuf>,
    recipients_files: Vec<PathBuf>,
    // TODO: maybe just for now
    flake_input: Option<String>,
}

impl SessionFile {
    pub fn new(
        source: PathBuf,
        template: Vec<u8>,
        file_name: OsString,
        target_dir: PathBuf,
        identity: PathBuf,
        recipients: Vec<PathBuf>,
        recipients_files: Vec<PathBuf>,
        flake_input: Option<String>,
    ) -> Result<Self> {
        let (path, dir) = Self::create_temp_file(&file_name)?;
        let target = target_dir.join(file_name);

        Ok(Self {
            path,
            dir,
            hash: None,
            content: template,
            source,
            target,
            identity,
            recipients,
            recipients_files,
            flake_input,
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

    pub fn flake_input(&self) -> Option<String> {
        self.flake_input.to_owned()
    }

    fn create_temp_file(name: &OsString) -> Result<(PathBuf, TempDir)> {
        let dir = TempDir::new()?;
        let path = dir.path().join(&name);

        Ok((path, dir))
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
