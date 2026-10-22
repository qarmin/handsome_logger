#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use time::UtcOffset;

use self::clock::{Clock, SystemClock};
use self::files::{free_timestamped_path, list_rotated, open_append, timestamp_suffix, with_suffix};
use crate::config::ConfigBuilder;

mod clock;
mod files;
#[cfg(test)]
mod tests;

const ROTATION_RETRY_DELAY_SECONDS: i64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPeriod {
    Hourly,
    Daily,
}

impl RotationPeriod {
    const fn seconds(self) -> i64 {
        match self {
            Self::Hourly => 3600,
            Self::Daily => 86400,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationType {
    /// File is only appended to, never rotated
    None,
    /// Rotates before a write that would make the file bigger than `max_bytes`
    /// A single message bigger than `max_bytes` is still written whole
    Size { max_bytes: u64 },
    /// Rotates with the first write after a new period begins
    Time { period: RotationPeriod },
    /// Rotates when any of the two conditions is met
    TimeSize { period: RotationPeriod, max_bytes: u64 },
}

impl RotationType {
    const fn period(self) -> Option<RotationPeriod> {
        match self {
            Self::Time { period } | Self::TimeSize { period, .. } => Some(period),
            Self::None | Self::Size { .. } => None,
        }
    }

    const fn max_bytes(self) -> Option<u64> {
        match self {
            Self::Size { max_bytes } | Self::TimeSize { max_bytes, .. } => Some(max_bytes),
            Self::None | Self::Time { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Naming {
    /// `app.log.20260422T143005`, with `.1`, `.2`... appended when such name already exists
    #[default]
    Timestamp,
    /// `app.log.1` is the newest rotated file, every rotation increases the numbers of older files
    Numbered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeZone {
    /// Local time, falls back to UTC when local offset cannot be determined
    #[default]
    Local,
    Utc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotationConfig {
    pub rotation: RotationType,
    pub naming: Naming,
    /// Number of rotated files to keep, `None` keeps all of them
    /// Only files matching the naming scheme of the log file are ever removed
    pub max_files: Option<usize>,
    /// Rotates already existing non-empty file with the first write after start
    pub rotate_on_startup: bool,
    /// Used for period boundaries and for timestamps in file names
    pub time_zone: TimeZone,
}

impl RotationConfig {
    #[must_use]
    pub const fn new(rotation: RotationType) -> Self {
        Self {
            rotation,
            naming: Naming::Timestamp,
            max_files: Some(10),
            rotate_on_startup: false,
            time_zone: TimeZone::Local,
        }
    }

    #[must_use]
    pub const fn naming(mut self, naming: Naming) -> Self {
        self.naming = naming;
        self
    }

    #[must_use]
    pub const fn max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }

    #[must_use]
    pub const fn keep_all(mut self) -> Self {
        self.max_files = None;
        self
    }

    #[must_use]
    pub const fn rotate_on_startup(mut self, rotate_on_startup: bool) -> Self {
        self.rotate_on_startup = rotate_on_startup;
        self
    }

    #[must_use]
    pub const fn time_zone(mut self, time_zone: TimeZone) -> Self {
        self.time_zone = time_zone;
        self
    }

    fn validate(&self) -> io::Result<()> {
        if self.rotation.max_bytes() == Some(0) {
            return Err(invalid_input("max_bytes must be greater than 0"));
        }
        if self.max_files == Some(0) {
            return Err(invalid_input("max_files must be greater than 0 or None"));
        }
        Ok(())
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self::new(RotationType::Size { max_bytes: 10 * 1024 * 1024 })
    }
}

fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

/// File writer rotating by size and/or time, intended to be used with `WriteLogger`
/// Never panics, when rotation fails it keeps appending to the current file and retries later
/// Rotation happens only between `write` calls, so use it with `single_write` (default) to keep messages whole
pub struct RotatingFile {
    path: PathBuf,
    base_name: String,
    config: RotationConfig,
    offset: UtcOffset,
    clock: Box<dyn Clock>,
    file: Option<File>,
    size: u64,
    period_key: i64,
    rotate_pending: bool,
    retry_after: i64,
}

impl RotatingFile {
    pub fn new(path: impl Into<PathBuf>, config: RotationConfig) -> io::Result<Self> {
        Self::with_clock(path.into(), config, Box::new(SystemClock))
    }

    pub(crate) fn with_clock(path: PathBuf, config: RotationConfig, clock: Box<dyn Clock>) -> io::Result<Self> {
        config.validate()?;
        let base_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid_input("log path must end with a valid UTF-8 file name"))?
            .to_string();
        let offset = match config.time_zone {
            TimeZone::Local => ConfigBuilder::get_local_time_offset().unwrap_or(UtcOffset::UTC),
            TimeZone::Utc => UtcOffset::UTC,
        };

        let file = open_append(&path)?;
        let metadata = file.metadata()?;
        let now = clock.now_unix();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|duration| i64::try_from(duration.as_secs()).ok())
            .filter(|_| metadata.len() > 0)
            .unwrap_or(now);

        let mut rotating = Self {
            path,
            base_name,
            config,
            offset,
            clock,
            file: Some(file),
            size: metadata.len(),
            period_key: 0,
            rotate_pending: config.rotate_on_startup,
            retry_after: 0,
        };
        rotating.period_key = rotating.period_key_of(modified);
        Ok(rotating)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn period_key_of(&self, unix: i64) -> i64 {
        match self.config.rotation.period() {
            Some(period) => unix.saturating_add(i64::from(self.offset.whole_seconds())).div_euclid(period.seconds()),
            None => 0,
        }
    }

    fn rotation_due(&self, now: i64, incoming: u64) -> bool {
        let size_exceeded = self.config.rotation.max_bytes().is_some_and(|max_bytes| self.size.saturating_add(incoming) > max_bytes);
        let period_elapsed = self.config.rotation.period().is_some() && self.period_key_of(now) != self.period_key;
        self.rotate_pending || size_exceeded || period_elapsed
    }

    fn rotate_if_due(&mut self, incoming: u64) {
        let now = self.clock.now_unix();
        if !self.rotation_due(now, incoming) {
            return;
        }
        if self.size == 0 {
            self.rotate_pending = false;
            self.period_key = self.period_key_of(now);
            return;
        }
        if now < self.retry_after {
            return;
        }
        self.rotate_pending = false;
        if self.rotate(now).is_err() {
            self.retry_after = now.saturating_add(ROTATION_RETRY_DELAY_SECONDS);
        }
    }

    fn rotate(&mut self, now: i64) -> io::Result<()> {
        self.file = None;
        let moved = self.move_current_aside(now);
        if moved.is_ok() {
            self.size = 0;
            self.period_key = self.period_key_of(now);
        }
        self.file = Some(open_append(&self.path)?);
        moved?;
        // Cleanup is best effort, failing to remove an old file must not be treated as failed rotation
        let _ = self.remove_excess_timestamped();
        Ok(())
    }

    fn move_current_aside(&self, now: i64) -> io::Result<()> {
        match self.config.naming {
            Naming::Timestamp => {
                let stamp = timestamp_suffix(now, self.offset)?;
                fs::rename(&self.path, free_timestamped_path(&self.path, &stamp)?)
            }
            Naming::Numbered => {
                let mut rotated = list_rotated(&self.path, &self.base_name, Naming::Numbered)?;
                rotated.reverse();
                if let Some(limit) = self.config.max_files {
                    let limit = u64::try_from(limit).unwrap_or(u64::MAX);
                    while let Some((path, index)) = rotated.first() {
                        if *index < limit {
                            break;
                        }
                        fs::remove_file(path)?;
                        rotated.remove(0);
                    }
                }
                for (path, index) in &rotated {
                    fs::rename(path, with_suffix(&self.path, &index.saturating_add(1).to_string()))?;
                }
                fs::rename(&self.path, with_suffix(&self.path, "1"))
            }
        }
    }

    fn remove_excess_timestamped(&self) -> io::Result<()> {
        let (Naming::Timestamp, Some(max_files)) = (self.config.naming, self.config.max_files) else {
            return Ok(());
        };
        let rotated = list_rotated(&self.path, &self.base_name, Naming::Timestamp)?;
        let excess = rotated.len().saturating_sub(max_files);
        for (path, _) in rotated.into_iter().take(excess) {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn file(&mut self) -> io::Result<&mut File> {
        if self.file.is_none() {
            let file = open_append(&self.path)?;
            self.size = file.metadata()?.len();
            self.file = Some(file);
        }
        self.file.as_mut().ok_or_else(|| io::Error::other("log file is not open"))
    }
}

impl Write for RotatingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let incoming = u64::try_from(buf.len()).unwrap_or(u64::MAX);
        self.rotate_if_due(incoming);
        let result = self.file().and_then(|file| file.write_all(buf));
        if let Err(error) = result {
            self.file = None;
            return Err(error);
        }
        self.size = self.size.saturating_add(incoming);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.file.as_mut() {
            Some(file) => file.flush(),
            None => Ok(()),
        }
    }
}
