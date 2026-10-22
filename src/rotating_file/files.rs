use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

use super::Naming;

const MAX_COLLISION_COUNTER: u32 = 10_000;

pub(super) fn open_append(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    OpenOptions::new().create(true).append(true).open(path)
}

pub(super) fn timestamp_suffix(now_unix: i64, offset: UtcOffset) -> io::Result<String> {
    let invalid = |error: &dyn std::fmt::Display| io::Error::new(io::ErrorKind::InvalidData, error.to_string());
    OffsetDateTime::from_unix_timestamp(now_unix)
        .map_err(|error| invalid(&error))?
        .to_offset(offset)
        .format(format_description!("[year][month][day]T[hour][minute][second]"))
        .map_err(|error| invalid(&error))
}

pub(super) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

pub(super) fn free_timestamped_path(path: &Path, stamp: &str) -> io::Result<PathBuf> {
    let first = with_suffix(path, stamp);
    if !first.try_exists()? {
        return Ok(first);
    }
    for counter in 1..=MAX_COLLISION_COUNTER {
        let candidate = with_suffix(path, &format!("{stamp}.{counter}"));
        if !candidate.try_exists()? {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "no free name for rotated log file"))
}

/// Rotated files of `path` created with the given naming, oldest first
pub(super) fn list_rotated(path: &Path, base_name: &str, naming: Naming) -> io::Result<Vec<(PathBuf, u64)>> {
    let directory = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let prefix = format!("{base_name}.");

    let mut found: Vec<((String, u64), PathBuf)> = Vec::new();
    for entry in fs::read_dir(directory)? {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_ok_and(|file_type| file_type.is_file()) {
            continue;
        }
        let file_name = entry.file_name();
        let Some(suffix) = file_name.to_str().and_then(|name| name.strip_prefix(&prefix)) else {
            continue;
        };
        let key = match naming {
            Naming::Timestamp => parse_timestamp_suffix(suffix).map(|(stamp, counter)| (stamp.to_string(), counter)),
            Naming::Numbered => suffix
                .parse::<u64>()
                .ok()
                .filter(|_| suffix.bytes().all(|byte| byte.is_ascii_digit()))
                .map(|index| (String::new(), index)),
        };
        if let Some(key) = key {
            found.push((key, entry.path()));
        }
    }
    found.sort();
    Ok(found.into_iter().map(|((_, order), path)| (path, order)).collect())
}

fn parse_timestamp_suffix(suffix: &str) -> Option<(&str, u64)> {
    let (stamp, counter) = match suffix.split_once('.') {
        Some((stamp, counter)) => (stamp, counter.parse().ok()?),
        None => (suffix, 0),
    };
    let (date, time) = stamp.split_once('T')?;
    let all_digits = |part: &str, length: usize| part.len() == length && part.bytes().all(|byte| byte.is_ascii_digit());
    (all_digits(date, 8) && all_digits(time, 6)).then_some((stamp, counter))
}
