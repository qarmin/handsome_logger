use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::common::get_env_log;
use crate::{Config, SharedLogger};

pub struct CombinedLogger {
    level: LevelFilter,
    logger: Vec<Box<dyn SharedLogger>>,
}

impl CombinedLogger {
    pub fn init(logger: Vec<Box<dyn SharedLogger>>) -> Result<(), SetLoggerError> {
        let comblog = Self::new(logger);
        let log_level = get_env_log().unwrap_or(comblog.level());
        set_max_level(log_level);
        set_boxed_logger(comblog)
    }

    #[must_use]
    pub fn new(logger: Vec<Box<dyn SharedLogger>>) -> Box<Self> {
        let mut log_level = LevelFilter::Off;
        for log in &logger {
            if log_level < log.level() {
                log_level = log.level();
            }
        }
        let log_level = get_env_log().unwrap_or(log_level);

        Box::new(Self { level: log_level, logger })
    }
}

impl Log for CombinedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            for log in &self.logger {
                log.log(record);
            }
        }
    }

    fn flush(&self) {
        for log in &self.logger {
            log.flush();
        }
    }
}

impl SharedLogger for CombinedLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
