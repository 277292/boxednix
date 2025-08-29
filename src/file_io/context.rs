// SPDX-License-Identifier: GPL-3.0-only

use blake3::Hash;

use crate::Result;

pub trait ReadingContext {
    fn input(&self) -> Option<String>;
    fn output(&mut self, content: Vec<u8>, hash: Hash);
}

pub trait WritingContext {
    fn input(&self) -> &[u8];
    fn output(&self) -> Option<String>;
    fn result(&mut self, hash: Hash);
}

pub trait DecryptionContext {
    fn input(&self) -> Option<String>;
    fn output(&self) -> Option<String>;
    fn identities(&self) -> Vec<String>;
    fn result(&mut self, hash: Hash);
}

pub trait EncryptionContext {
    fn input(&self) -> &[u8];
    fn output(&self) -> Option<String>;
    fn identities(&self) -> Vec<String>;
    fn recipients(&self) -> Vec<String>;
    fn recipients_files(&self) -> Vec<String>;
}

pub trait GenerationContext {
    fn input(&self) -> Result<Vec<u8>>;
    fn output(&self) -> Option<String>;
}
