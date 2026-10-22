use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) trait Clock: Send {
    fn now_unix(&self) -> i64;
}

pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
    }
}
