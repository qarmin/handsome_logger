use log::LevelFilter;
use std::env;

pub fn get_env_log() -> Option<LevelFilter> {
    match env::var("RUST_LOG").ok()?.as_str() {
        "err" | "error" => Some(LevelFilter::Error),
        "warn" | "warning" => Some(LevelFilter::Warn),
        "info" => Some(LevelFilter::Info),
        "debug" => Some(LevelFilter::Debug),
        "trace" => Some(LevelFilter::Trace),
        _ => None,
    }
}
