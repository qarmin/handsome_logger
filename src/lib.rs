pub use log::{Level, LevelFilter, Log};
pub use termcolor::{Color, ColorChoice};

pub use self::config::{format_description, Config, ConfigBuilder, FormatItem, TimeFormat};
pub use self::loggers::{CombinedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger};

mod common;
mod config;
mod loggers;

pub trait SharedLogger: Log {
    fn level(&self) -> LevelFilter;
    fn config(&self) -> Option<&Config>;
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
