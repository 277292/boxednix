// SPDX-License-Identifier: GPL-3.0-only

mod context;
mod hash;
mod identity;

use age::{
    armor::{ArmoredReader, ArmoredWriter, Format},
    cli_common::file_io::{InputReader, OutputFormat, OutputWriter},
    Decryptor, Encryptor, Identity,
};
use anyhow::anyhow;
use hash::{HashingReader, HashingWriter};
use std::{
    io::{self, Read, Write},
    path::Path,
};

use crate::Result;
use identity::{read_identities, read_recipients};

pub(crate) use context::{
    DecryptionContext, EncryptionContext, GenerationContext, ReadingContext, WritingContext,
};
pub(crate) use identity::create_identity;

pub fn read<R: ReadingContext>(ctx: &mut R) -> Result<()> {
    let mut input = HashingReader::new(InputReader::new(ctx.input())?);
    let (content, hash) = input.read_all()?;
    ctx.output(content, hash);
    Ok(())
}

pub fn encrypt<E: EncryptionContext>(ctx: &E) -> Result<()> {
    let output = OutputWriter::new(ctx.output(), true, OutputFormat::Unknown, 0o666, false)?;
    let output = ArmoredWriter::wrap_output(output, Format::AsciiArmor)?;
    let recipients = read_recipients(ctx.recipients(), ctx.recipients_files(), ctx.identities())?;
    let encryptor = Encryptor::with_recipients(recipients.iter().map(|r| r.as_ref() as _))?;
    let mut output = encryptor.wrap_output(output)?;

    output.write_all(ctx.input())?;
    output.finish().and_then(|armor| armor.finish())?;
    Ok(())
}

pub fn generate<G: GenerationContext>(ctx: &G) -> Result<()> {
    create_parent(ctx.output())?;
    let mut output = OutputWriter::new(ctx.output(), true, OutputFormat::Text, 0o666, false)?;
    output.write_all(&ctx.input()?)?;
    output.flush()?;
    Ok(())
}

pub fn decrypt<D: DecryptionContext>(ctx: &mut D) -> Result<()> {
    let input = InputReader::new(ctx.input())?;
    let output = OutputWriter::new(ctx.output(), true, OutputFormat::Unknown, 0o666, false)?;

    let decryptor = Decryptor::new_buffered(ArmoredReader::new(input))?;
    let mut output = HashingWriter::new(output);

    let identities = ctx.identities();
    if identities.is_empty() {
        return Err(anyhow!("No identities"));
    }
    let identities = read_identities(identities)?;

    decryptor
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn Identity))
        .map_err(|e| e.into())
        .and_then(|input| copy(input, &mut output))?;

    let hash = output.finalize();
    ctx.result(hash);
    Ok(())
}

pub fn write<W: WritingContext>(ctx: &mut W) -> Result<()> {
    let output = OutputWriter::new(ctx.output(), true, OutputFormat::Text, 0o666, false)?;
    let mut output = HashingWriter::new(output);
    output.write_all(ctx.input())?;
    output.flush()?;
    let hash = output.finalize();
    ctx.result(hash);
    Ok(())
}

fn copy<R: Read, W: Write>(mut input: R, mut output: W) -> Result<()> {
    io::copy(&mut input, &mut output)?;

    Ok(())
}

// TODO: just for now
fn create_parent(path: Option<String>) -> Result<()> {
    let Some(path) = path else { return Ok(()) };
    let Some(parent) = Path::new(&path).parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    Ok(())
}
