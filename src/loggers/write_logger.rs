use std::io::Write;
use std::sync::Mutex;

use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::{Config, SharedLogger};

use super::logging::try_log;

pub struct WriteLogger<W: Write + Send + 'static> {
    level: LevelFilter,
    config: Config,
    writable: Mutex<W>,
}

impl<W: Write + Send + 'static> WriteLogger<W> {
    pub fn init(config: Config, writable: W) -> Result<(), SetLoggerError> {
        set_max_level(config.level);
        set_boxed_logger(WriteLogger::new(config, writable))
    }

    #[must_use]
    pub fn new(mut config: Config, writable: W) -> Box<WriteLogger<W>> {
        config.calculate_data();

        Box::new(WriteLogger {
            level: config.level,
            config,
            writable: Mutex::new(writable),
        })
    }
}

impl<W: Write + Send + 'static> Log for WriteLogger<W> {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let mut write_lock = self.writable.lock().unwrap();
            let _ = try_log(&self.config, record, &mut *write_lock);
        }
    }

    fn flush(&self) {
        let _ = self.writable.lock().unwrap().flush();
    }
}

impl<W: Write + Send + 'static> SharedLogger for WriteLogger<W> {
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
