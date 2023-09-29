pub use log::{Level, LevelFilter, Log};
pub use termcolor::{Color, ColorChoice};

pub use self::config::{format_description, Config, ConfigBuilder, FormatItem, TimeFormat};
pub use self::loggers::{CombinedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger};

mod common;
mod config;
mod loggers;

pub fn init() -> Result<(), log::SetLoggerError> {
    TermLogger::init(Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
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
