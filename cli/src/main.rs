mod cli;
mod commands;
mod config;
mod templates;
mod utils;

use crate::cli::{Cli, Commands};
use clap::Parser;
use config::{ConfigProvider, EnvConfigProvider};
use log::info;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let conf = EnvConfigProvider::load_config().expect("Unable to load environment variables");

    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { path }) => {
            commands::init::execute(path)?;
        }
        Some(Commands::Hydrate { openapi_file }) => {
            commands::hydrate::execute(openapi_file, &conf)?;
        }
        Some(Commands::Generate {
            openapi_file,
            all,
            lib,
            manifests,
            controllers,
            types,
        }) => {
            commands::generate::execute(openapi_file, all, lib, manifests, controllers, types)?;
        }
        None => {
            info!("No command provided");
        }
    }

    Ok(())
}
