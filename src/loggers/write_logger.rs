use std::io::Write;
use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError, set_boxed_logger, set_max_level};

use crate::common::get_env_log;
use crate::{Config, RotatingFile, SharedLogger};

use super::logging::try_log;

pub struct WriteLogger<W: Write + Send + 'static> {
    level: LevelFilter,
    config: Config,
    writable: Mutex<W>,
}

impl<W: Write + Send + 'static> WriteLogger<W> {
    pub fn init(config: Config, writable: W) -> Result<(), SetLoggerError> {
        let log_level = get_env_log().unwrap_or(config.level);
        set_max_level(log_level);
        let logger = Self::new(config, writable);
        set_boxed_logger(logger)
    }

    #[must_use]
    pub fn new(mut config: Config, writable: W) -> Box<Self> {
        config.calculate_data();

        let log_level = get_env_log().unwrap_or(config.level);
        Box::new(Self {
            level: log_level,
            config,
            writable: Mutex::new(writable),
        })
    }
}

impl WriteLogger<RotatingFile> {
    /// Same as `init`, but forces `single_write`, so one message is never split between two files
    pub fn init_rotating(config: Config, file: RotatingFile) -> Result<(), SetLoggerError> {
        let log_level = get_env_log().unwrap_or(config.level);
        set_max_level(log_level);
        set_boxed_logger(Self::new_rotating(config, file))
    }

    /// Same as `new`, but forces `single_write`, so one message is never split between two files
    #[must_use]
    pub fn new_rotating(mut config: Config, file: RotatingFile) -> Box<Self> {
        config.single_write = true;
        Self::new(config, file)
    }
}

impl<W: Write + Send + 'static> Log for WriteLogger<W> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.config.is_record_allowed(record) {
            return;
        }

        if self.enabled(record.metadata()) {
            let mut write_lock = self.writable.lock().unwrap();

            if let Some(write_formatter) = &self.config.write_formatter {
                if self.config.single_write {
                    let mut buffer: Vec<u8> = Vec::new();
                    let _ = write_formatter(record, &mut buffer);
                    let _ = write_lock.write_all(buffer.as_slice());
                } else {
                    let _ = write_formatter(record, &mut *write_lock);
                }
            } else if self.config.single_write {
                let mut buffer: Vec<u8> = Vec::new();
                let _ = try_log(&self.config, record, &mut buffer);
                let _ = write_lock.write_all(buffer.as_slice());
            } else {
                let _ = try_log(&self.config, record, &mut *write_lock);
            }
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
