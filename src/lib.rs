use log::Log;
pub use log::{Level, LevelFilter};
pub use termcolor::{Color, ColorChoice};

pub use self::config::{format_description, Config, ConfigBuilder, FormatItem};
pub use self::loggers::{CombinedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger};

mod config;
mod loggers;

pub trait SharedLogger: Log {
    fn level(&self) -> LevelFilter;
    fn config(&self) -> Option<&Config>;
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
