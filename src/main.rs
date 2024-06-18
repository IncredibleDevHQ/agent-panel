mod client;
mod config;
mod function;
mod logger;
mod serve;
#[macro_use]
mod utils;

#[macro_use]
extern crate log;

use crate::config::Config;
use crate::utils::{create_abort_signal, CODE_BLOCK_RE, IS_STDOUT_TERMINAL};

use anyhow::{bail, Result};
use clap::Parser;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port number to run the server on (optional)
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    crate::logger::setup_logger()?;
    let config = Arc::new(RwLock::new(Config::init()?));

    return serve::run(config, args.port).await;
}
