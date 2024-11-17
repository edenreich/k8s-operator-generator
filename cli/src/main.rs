mod cli;
mod commands;
mod config;
mod errors;
mod templates;
mod utils;

use crate::{
    cli::{Cli, Commands},
    config::{Config, ConfigProvider},
    errors::AppError,
};
use clap::Parser;
use log::info;

fn main() -> Result<(), AppError> {
    dotenv::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();
    let conf = Config::load_from_cli(&cli)?;

    println!("{:?}", conf);

    match &cli.command {
        Some(Commands::Init { path }) => {
            commands::init::execute(conf, path)?;
        }
        Some(Commands::Hydrate { openapi_file }) => {
            commands::hydrate::execute(conf, openapi_file)?;
        }
        Some(Commands::Generate {
            openapi_file,
            path,
            all,
            lib,
            manifests,
            controllers,
            types,
        }) => {
            commands::generate::execute(
                path,
                openapi_file,
                all,
                lib,
                manifests,
                controllers,
                types,
            )?;
        }
        None => {
            info!("No command provided");
        }
    }

    Ok(())
}
