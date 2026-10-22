#![allow(clippy::collapsible_if)]
pub use log::{Level, LevelFilter, Log};
pub use termcolor::{Color, ColorChoice};

pub use self::config::{Config, ConfigBuilder, FormatItem, FormatText, TimeFormat, format_description};
pub use self::loggers::{CombinedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger};
pub use self::rotating_file::{Naming, RotatingFile, RotationConfig, RotationPeriod, RotationType, TimeZone};
pub use self::setup::{ActiveOutputs, CombinedLoggerBuilder, InitError};

mod common;
mod config;
mod loggers;
mod rotating_file;
mod setup;

pub fn init() -> Result<(), log::SetLoggerError> {
    TermLogger::init(Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
}

/// Logs to terminal (from `Info`) and to the rotating file (from `Debug`)
pub fn init_with_file(path: impl Into<std::path::PathBuf>, rotation: RotationType) -> Result<(), InitError> {
    CombinedLogger::builder()
        .terminal(Config::default())
        .file(ConfigBuilder::for_file().build(), path, RotationConfig::new(rotation))
        .init()
        .map(|_| ())
}

pub fn init_without_local_time() -> Result<(), log::SetLoggerError> {
    let config = ConfigBuilder::default().set_remove_time_offset().build();
    TermLogger::init(config, TerminalMode::Mixed, ColorChoice::Auto)
}

pub trait SharedLogger: Log {
    fn level(&self) -> LevelFilter;
    fn config(&self) -> Option<&Config>;
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
