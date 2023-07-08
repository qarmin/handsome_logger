use std::io::{stderr, stdout};
use std::sync::Mutex;

use log::{set_boxed_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::{Config, SharedLogger};

use super::logging::try_log;

pub struct SimpleLogger {
    level: LevelFilter,
    config: Config,
    output_lock: Mutex<()>,
}

impl SimpleLogger {
    pub fn init(config: Config) -> Result<(), SetLoggerError> {
        set_max_level(config.level);
        set_boxed_logger(SimpleLogger::new(config.level, config))
    }

    #[must_use]
    pub fn new(log_level: LevelFilter, mut config: Config) -> Box<SimpleLogger> {
        config.calculate_data();

        Box::new(SimpleLogger {
            level: log_level,
            config,
            output_lock: Mutex::new(()),
        })
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if let Some(message_filtering) = &self.config.message_filtering {
            if !message_filtering(record) {
                return;
            }
        }

        if self.enabled(record.metadata()) {
            let _lock = self.output_lock.lock().unwrap();

            if record.level() == Level::Error {
                let stderr = stderr();
                let mut stderr_lock = stderr.lock();
                let _ = try_log(&self.config, record, &mut stderr_lock);
            } else {
                let stdout = stdout();
                let mut stdout_lock = stdout.lock();
                let _ = try_log(&self.config, record, &mut stdout_lock);
            }
        }
    }

    fn flush(&self) {
        use std::io::Write;
        let _ = stdout().flush();
    }
}

impl SharedLogger for SimpleLogger {
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
