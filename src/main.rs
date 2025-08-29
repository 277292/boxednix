// SPDX-License-Identifier: GPL-3.0-only

mod cli;

pub(crate) use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
