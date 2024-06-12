mod cli;
mod client;
mod config;
mod function;
mod logger;
mod serve;
#[macro_use]
mod utils;

#[macro_use]
extern crate log;

use crate::cli::Cli;
use crate::config::{
    Config, GlobalConfig, Input, InputContext, WorkingMode,
};
use crate::function::{eval_tool_calls, need_send_call_results};
use crate::utils::{
    create_abort_signal, detect_shell, extract_block, run_command,  Shell,
    CODE_BLOCK_RE, IS_STDOUT_TERMINAL,
};

use anyhow::{bail, Result};
use async_recursion::async_recursion;
use clap::Parser;
use inquire::{Select, Text};
use parking_lot::RwLock;
use std::io::{stderr, stdin, Read};
use std::process;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    crate::logger::setup_logger()?;
    let config = Arc::new(RwLock::new(Config::init()?));

    if let Some(addr) = cli.serve {
        return serve::run(config, addr).await;
    }
    
    Ok(())
}
