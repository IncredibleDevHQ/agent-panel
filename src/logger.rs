use crate::config::WorkingMode;

use anyhow::Result;
use log::LevelFilter;
use simplelog::{format_description, Config as LogConfig, ConfigBuilder};

#[cfg(debug_assertions)]
pub fn setup_logger(working_mode: WorkingMode) -> Result<()> {
    let config = build_config();
        simplelog::SimpleLogger::init(LevelFilter::Debug, config)?;
    Ok(())
}

#[cfg(not(debug_assertions))]
pub fn setup_logger(working_mode: WorkingMode) -> Result<()> {
    let config = build_config();
    if working_mode == WorkingMode::Serve {
        simplelog::SimpleLogger::init(log::LevelFilter::Info, config)?;
    }
    Ok(())
}

fn build_config() -> LogConfig {
    ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
        ))
        .set_thread_level(LevelFilter::Off)
        .build()
}
