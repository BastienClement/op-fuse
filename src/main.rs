#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;

mod config;
mod fs;
mod onepassword;
mod util;

use std::path::Path;

pub use config::Config;
pub use onepassword::OnePassword;

use anyhow::Result;
use clap::Parser;
use fuser::MountOption;
use tracing::debug;
use tracing_subscriber::prelude::*;

#[derive(Debug, Parser)]
#[clap(version)]
/// Mount 1Password vaults as a filesystem
struct Cli {
    /// The configuration file to read
    config: String,

    /// Whether to allow other users to access the filesystem
    #[clap(long)]
    allow_others: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    let cli = Cli::parse();

    let config = Config::read(Path::new(&cli.config))?;
    debug!(config = ?config);

    let op = OnePassword::new(&config);
    let filesystem = fs::Fs::new(&config, op);

    fuser::mount2(
        filesystem,
        &config.mountpoint,
        &[
            MountOption::AutoUnmount,
            if cli.allow_others {
                MountOption::AllowOther
            } else {
                MountOption::AllowRoot
            },
            MountOption::DefaultPermissions,
        ],
    )?;

    Ok(())
}
