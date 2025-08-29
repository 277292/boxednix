// SPDX-License-Identifier: GPL-3.0-only

use blake3::{Hash, Hasher};
use std::io::{Read, Result, Write};

pub struct HashingWriter<W: Write> {
    inner: W,
    hasher: Hasher,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.hasher.update(buf);
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> HashingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: Hasher::new(),
        }
    }
    pub fn finalize(self) -> Hash {
        self.hasher.finalize()
    }
}

pub struct HashingReader<R> {
    inner: R,
    hasher: Hasher,
}

impl<R: Read> HashingReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Hasher::new(),
        }
    }

    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut tmp = [0u8; 8192]; // 8 KiB Puffer
        let mut total = 0;

        loop {
            let n = self.read(&mut tmp)?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            total += n;
        }

        Ok(total)
    }

    pub fn read_all(&mut self) -> Result<(Vec<u8>, Hash)> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;
        Ok((buf, self.finalize()))
    }

    pub fn finalize(&self) -> Hash {
        self.hasher.finalize()
    }
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
}
