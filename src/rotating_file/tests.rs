#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use super::clock::Clock;
use super::*;

const START: i64 = 1_800_000_000;

#[derive(Clone)]
struct TestClock(Arc<AtomicI64>);

impl TestClock {
    fn new(start: i64) -> Self {
        Self(Arc::new(AtomicI64::new(start)))
    }

    fn advance(&self, seconds: i64) {
        self.0.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now_unix(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

fn test_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tmp").join("rotating_file_tests").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn utc(rotation: RotationType) -> RotationConfig {
    RotationConfig::new(rotation).time_zone(TimeZone::Utc)
}

fn open(path: &Path, config: RotationConfig, clock: &TestClock) -> RotatingFile {
    RotatingFile::with_clock(path.to_path_buf(), config, Box::new(clock.clone())).unwrap()
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    names
}

#[test]
fn size_rotation_with_timestamp_names() {
    let dir = test_dir("size_timestamp");
    let path = dir.join("app.log");
    let clock = TestClock::new(START);
    let mut file = open(&path, utc(RotationType::Size { max_bytes: 10 }), &clock);

    file.write_all(b"aaaaaa\n").unwrap();
    file.write_all(b"bbbbbb\n").unwrap();

    let names = file_names(&dir);
    assert_eq!(names, ["app.log", "app.log.20270115T080000"]);
    assert_eq!(fs::read_to_string(&path).unwrap(), "bbbbbb\n");
    assert_eq!(fs::read_to_string(dir.join("app.log.20270115T080000")).unwrap(), "aaaaaa\n");
}

#[test]
fn timestamp_collision_adds_counter() {
    let dir = test_dir("collision");
    let clock = TestClock::new(START);
    let mut file = open(&dir.join("app.log"), utc(RotationType::Size { max_bytes: 5 }).keep_all(), &clock);

    for _ in 0..4 {
        file.write_all(b"1234\n").unwrap();
    }

    assert_eq!(
        file_names(&dir),
        ["app.log", "app.log.20270115T080000", "app.log.20270115T080000.1", "app.log.20270115T080000.2"]
    );
}

#[test]
fn message_bigger_than_limit_is_written_whole() {
    let dir = test_dir("big_message");
    let path = dir.join("app.log");
    let clock = TestClock::new(START);
    let mut file = open(&path, utc(RotationType::Size { max_bytes: 3 }), &clock);

    file.write_all(b"much longer than limit\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "much longer than limit\n");
    assert_eq!(file_names(&dir), ["app.log"]);
}

#[test]
fn max_files_removes_oldest_timestamped() {
    let dir = test_dir("retention_timestamp");
    let clock = TestClock::new(START);
    let mut file = open(&dir.join("app.log"), utc(RotationType::Size { max_bytes: 5 }).max_files(2), &clock);

    for index in 0..5 {
        file.write_all(format!("{index}234\n").as_bytes()).unwrap();
        clock.advance(1);
    }

    assert_eq!(file_names(&dir), ["app.log", "app.log.20270115T080003", "app.log.20270115T080004"]);
    assert_eq!(fs::read_to_string(dir.join("app.log.20270115T080004")).unwrap(), "3234\n");
}

#[test]
fn numbered_naming_shifts_and_limits_files() {
    let dir = test_dir("numbered");
    let path = dir.join("app.log");
    let clock = TestClock::new(START);
    let config = utc(RotationType::Size { max_bytes: 5 }).naming(Naming::Numbered).max_files(2);
    let mut file = open(&path, config, &clock);

    for index in 0..5 {
        file.write_all(format!("{index}234\n").as_bytes()).unwrap();
    }

    assert_eq!(file_names(&dir), ["app.log", "app.log.1", "app.log.2"]);
    assert_eq!(fs::read_to_string(&path).unwrap(), "4234\n");
    assert_eq!(fs::read_to_string(dir.join("app.log.1")).unwrap(), "3234\n");
    assert_eq!(fs::read_to_string(dir.join("app.log.2")).unwrap(), "2234\n");
}

#[test]
fn only_matching_files_are_removed() {
    let dir = test_dir("foreign_files");
    for name in ["app.log.bak", "app.log.old.1", "app.log.123", "other.log.20270101T000000", "app.log.2027T1"] {
        fs::write(dir.join(name), "keep").unwrap();
    }
    fs::create_dir(dir.join("app.log.20270101T000000")).unwrap();
    let clock = TestClock::new(START);
    let mut file = open(&dir.join("app.log"), utc(RotationType::Size { max_bytes: 5 }).max_files(1), &clock);

    for index in 0..4 {
        file.write_all(format!("{index}234\n").as_bytes()).unwrap();
        clock.advance(1);
    }

    assert_eq!(
        file_names(&dir),
        [
            "app.log",
            "app.log.123",
            "app.log.20270101T000000",
            "app.log.20270115T080003",
            "app.log.2027T1",
            "app.log.bak",
            "app.log.old.1",
            "other.log.20270101T000000"
        ]
    );
}

#[test]
fn daily_rotation_after_midnight_only() {
    let dir = test_dir("daily");
    let path = dir.join("app.log");
    let midnight = START - START.rem_euclid(86400);
    let clock = TestClock::new(midnight + 3600);
    let mut file = open(&path, utc(RotationType::Time { period: RotationPeriod::Daily }), &clock);

    file.write_all(b"one\n").unwrap();
    clock.advance(20 * 3600);
    file.write_all(b"two\n").unwrap();
    assert_eq!(file_names(&dir), ["app.log"]);

    clock.advance(4 * 3600);
    file.write_all(b"three\n").unwrap();

    assert_eq!(file_names(&dir).len(), 2);
    assert_eq!(fs::read_to_string(&path).unwrap(), "three\n");
}

#[test]
fn hourly_rotation_and_time_size_rotation() {
    let dir = test_dir("time_size");
    let path = dir.join("app.log");
    let clock = TestClock::new(START - START.rem_euclid(3600));
    let config = utc(RotationType::TimeSize {
        period: RotationPeriod::Hourly,
        max_bytes: 12,
    });
    let mut file = open(&path, config, &clock);

    file.write_all(b"aaaaaa\n").unwrap();
    file.write_all(b"bbbbbb\n").unwrap();
    assert_eq!(file_names(&dir).len(), 2, "size triggered");

    clock.advance(3600);
    file.write_all(b"c\n").unwrap();
    assert_eq!(file_names(&dir).len(), 3, "time triggered");
    assert_eq!(fs::read_to_string(&path).unwrap(), "c\n");
}

#[test]
fn empty_file_is_not_rotated() {
    let dir = test_dir("empty");
    let clock = TestClock::new(START);
    let mut file = open(
        &dir.join("app.log"),
        utc(RotationType::Time { period: RotationPeriod::Hourly }).rotate_on_startup(true),
        &clock,
    );

    clock.advance(10 * 3600);
    file.write_all(b"x\n").unwrap();

    assert_eq!(file_names(&dir), ["app.log"]);
}

#[test]
fn existing_file_is_appended_and_size_restored() {
    let dir = test_dir("restart");
    let path = dir.join("app.log");
    fs::write(&path, "123456\n").unwrap();
    let clock = TestClock::new(START);
    let mut file = open(&path, utc(RotationType::Size { max_bytes: 12 }), &clock);

    file.write_all(b"abc\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "123456\nabc\n");

    file.write_all(b"abc\n").unwrap();
    assert_eq!(file_names(&dir).len(), 2);
    assert_eq!(fs::read_to_string(&path).unwrap(), "abc\n");
}

#[test]
fn rotate_on_startup_moves_existing_content() {
    let dir = test_dir("startup");
    let path = dir.join("app.log");
    fs::write(&path, "old\n").unwrap();
    let clock = TestClock::new(START);
    let mut file = open(&path, utc(RotationType::None).rotate_on_startup(true), &clock);

    file.write_all(b"new\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
    assert_eq!(fs::read_to_string(dir.join("app.log.20270115T080000")).unwrap(), "old\n");
}

#[test]
fn old_file_from_previous_period_is_rotated_at_start() {
    let dir = test_dir("previous_period");
    let path = dir.join("app.log");
    fs::write(&path, "yesterday\n").unwrap();
    let real_now = i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).unwrap();
    let clock = TestClock::new(real_now + 2 * 86400);
    let mut file = open(&path, utc(RotationType::Time { period: RotationPeriod::Daily }), &clock);

    file.write_all(b"today\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "today\n");
    assert_eq!(file_names(&dir).len(), 2);
}

#[test]
fn rotation_type_none_never_rotates() {
    let dir = test_dir("none");
    let clock = TestClock::new(START);
    let mut file = open(&dir.join("app.log"), utc(RotationType::None), &clock);

    for _ in 0..100 {
        file.write_all(b"line\n").unwrap();
        clock.advance(86400);
    }

    assert_eq!(file_names(&dir), ["app.log"]);
}

#[test]
fn creates_missing_directories() {
    let dir = test_dir("create_dirs");
    let path = dir.join("a").join("b").join("app.log");
    let mut file = RotatingFile::new(&path, RotationConfig::default()).unwrap();

    file.write_all(b"x\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "x\n");
}

#[test]
fn invalid_configs_are_rejected() {
    let dir = test_dir("invalid");
    let path = dir.join("app.log");

    let zero_bytes = RotatingFile::new(&path, RotationConfig::new(RotationType::Size { max_bytes: 0 }));
    assert_eq!(zero_bytes.err().unwrap().kind(), ErrorKind::InvalidInput);
    let zero_files = RotatingFile::new(&path, RotationConfig::default().max_files(0));
    assert_eq!(zero_files.err().unwrap().kind(), ErrorKind::InvalidInput);
    let no_name = RotatingFile::new("/", RotationConfig::default());
    assert!(no_name.is_err());
}

#[test]
fn failed_rotation_keeps_appending_and_retries_later() {
    let dir = test_dir("failed_rotation");
    let path = dir.join("app.log");
    fs::create_dir(dir.join("app.log.1")).unwrap();
    let clock = TestClock::new(START);
    let config = utc(RotationType::Size { max_bytes: 5 }).naming(Naming::Numbered);
    let mut file = open(&path, config, &clock);

    file.write_all(b"1234\n").unwrap();
    file.write_all(b"5678\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "1234\n5678\n");

    fs::remove_dir(dir.join("app.log.1")).unwrap();
    file.write_all(b"9999\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "1234\n5678\n9999\n", "retry is delayed");

    clock.advance(ROTATION_RETRY_DELAY_SECONDS);
    file.write_all(b"last\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "last\n");
    assert_eq!(fs::read_to_string(dir.join("app.log.1")).unwrap(), "1234\n5678\n9999\n");
}

#[test]
fn recovers_when_log_directory_disappears() {
    let dir = test_dir("dir_removed");
    let path = dir.join("logs").join("app.log");
    let clock = TestClock::new(START);
    let mut file = open(&path, utc(RotationType::Size { max_bytes: 5 }), &clock);
    file.write_all(b"1234\n").unwrap();

    fs::remove_dir_all(dir.join("logs")).unwrap();
    file.write_all(b"5678\n").unwrap();

    assert!(path.exists());
}
