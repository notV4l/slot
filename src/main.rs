mod browser;
mod cli;
mod command;
mod server;

use clap::Parser;
use cli::Cli;
use log::error;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command.run().await {
        Ok(_) => {}
        Err(e) => {
            error!("{e}")
        }
    }
}
