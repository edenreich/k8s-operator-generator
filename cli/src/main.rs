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
use dotenvy::dotenv;
use log::{debug, info};

fn main() -> Result<(), AppError> {
    dotenv().ok();

    let cli = Cli::parse();
    let conf = Config::load_from_cli(&cli)?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    if cli.verbosity != "info" {
        std::env::remove_var("RUST_LOG");
        std::env::set_var("RUST_LOG", &cli.verbosity);
    }

    env_logger::init();

    debug!(
        "Running in debug mode. These are the configurations:\n {:#?}",
        conf
    );

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
            manifests,
            controllers,
            types,
        }) => {
            commands::generate::execute(path, openapi_file, all, manifests, controllers, types)?;
        }
        None => {
            info!("No command provided");
        }
    }

    Ok(())
}
