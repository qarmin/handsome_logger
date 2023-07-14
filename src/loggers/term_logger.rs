use std::io::{Error, Write};
use std::sync::Mutex;

use log::{set_boxed_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use termcolor::{BufferedStandardStream, ColorChoice};

use crate::{Config, SharedLogger};

use super::logging::*;

pub struct OutputStreams {
    pub(crate) err: BufferedStandardStream,
    pub(crate) out: BufferedStandardStream,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum TerminalMode {
    Stdout,
    Stderr,
    #[default]
    Mixed,
}

pub struct TermLogger {
    level: LevelFilter,
    config: Config,
    streams: Mutex<OutputStreams>,
}

impl TermLogger {
    pub fn init(config: Config, mode: TerminalMode, color_choice: ColorChoice) -> Result<(), SetLoggerError> {
        set_max_level(config.level);
        let logger = TermLogger::new(config, mode, color_choice);
        set_boxed_logger(logger)?;
        Ok(())
    }

    #[must_use]
    pub fn new(mut config: Config, mode: TerminalMode, color_choice: ColorChoice) -> Box<TermLogger> {
        let streams = match mode {
            TerminalMode::Stdout => OutputStreams {
                err: BufferedStandardStream::stdout(color_choice),
                out: BufferedStandardStream::stdout(color_choice),
            },
            TerminalMode::Stderr => OutputStreams {
                err: BufferedStandardStream::stderr(color_choice),
                out: BufferedStandardStream::stderr(color_choice),
            },
            TerminalMode::Mixed => OutputStreams {
                err: BufferedStandardStream::stderr(color_choice),
                out: BufferedStandardStream::stdout(color_choice),
            },
        };

        config.calculate_data();

        Box::new(TermLogger {
            level: config.level,
            config,
            streams: Mutex::new(streams),
        })
    }

    fn try_log(&self, record: &Record) -> Result<(), Error> {
        if self.enabled(record.metadata()) {
            let mut streams = self.streams.lock().unwrap();

            if let Some(terminal_logger) = &self.config.terminal_formatter {
                if record.level() == Level::Error {
                    terminal_logger(record, &mut streams.err)
                } else {
                    terminal_logger(record, &mut streams.out)
                }
            } else if record.level() == Level::Error {
                try_log_term(&self.config, record, &mut streams.err)
            } else {
                try_log_term(&self.config, record, &mut streams.out)
            }
        } else {
            Ok(())
        }
    }
}

impl Log for TermLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if let Some(message_filtering) = &self.config.message_filtering {
            if !message_filtering(record) {
                return;
            }
        }

        let _ = self.try_log(record);
    }

    fn flush(&self) {
        let mut streams = self.streams.lock().unwrap();
        let _ = streams.out.flush();
        let _ = streams.err.flush();
    }
}

impl SharedLogger for TermLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
