// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use log::error;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
use std::{env, fs::File, io, path::PathBuf};

use crate::Result;

#[derive(Parser)]
#[command(
    author, version, about, long_about = None,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_value_name = "SUBCOMMAND",
    subcommand_help_heading = "Subcommands"
)]
struct Cli {
    #[arg(required = true)]
    pub source: Option<PathBuf>,

    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub editor: Option<String>,

    #[arg(trailing_var_arg = true, requires = "editor")]
    pub editor_args: Vec<String>,

    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub subcommand: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    New {
        identity: PathBuf,

        #[arg(short, long)]
        dir: Option<PathBuf>,

        #[arg(short, long)]
        passphrase: bool,

        #[arg(short, long)]
        recipients: Vec<PathBuf>,

        #[arg(short = 'R', long)]
        recipients_files: Vec<PathBuf>,
    },
    Copy {
        source: PathBuf,
        target: Option<PathBuf>,
    },
    Completions {
        shell: Shell,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.debug {
        init_logger("boxednix.log")?;
    }

    match handle_cli(cli) {
        Ok(_) => return Ok(()),
        Err(error) => {
            error!("{:?}", error);
            return Err(error);
        }
    }
}

fn handle_cli(cli: Cli) -> Result<()> {
    if let Some(subcommand) = cli.subcommand {
        match subcommand {
            Sub::Completions { shell } => {
                clap_complete::generate(
                    shell,
                    &mut Cli::command(),
                    env!("CARGO_PKG_NAME"),
                    &mut io::stdout(),
                );

                return Ok(());
            }
            Sub::New {
                identity,
                dir,
                recipients,
                recipients_files,
                passphrase,
            } => {
                return boxednix::create_config(
                    identity,
                    dir,
                    passphrase,
                    recipients,
                    recipients_files,
                )
            }
            Sub::Copy {
                source: _,
                target: _,
            } => return Err(anyhow!("Not yet...")),
        }
    }

    let source = cli.source.ok_or(anyhow!("Source are invalid"))?;
    let editor = match cli.editor {
        Some(editor) => editor,
        None => env::var("EDITOR")?,
    };

    boxednix::run(source, &editor, &cli.editor_args)?;

    Ok(())
}

fn init_logger(log_path: &str) -> Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create(log_path)?,
    )])?;
    Ok(())
}
