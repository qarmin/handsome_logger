use handsome_logger::{CombinedLogger, Config, RotationConfig, RotationPeriod, RotationType, init_with_file};
use log::*;

fn main() {
    // Terminal (from Info) and file (from Debug), the file is rotated every day
    init_with_file("my_rust_binary.log", RotationType::Time { period: RotationPeriod::Daily }).unwrap();
    info!("Got INFO");

    // Builder allows to customize each output and to continue with terminal only, when file cannot be opened:
    let _ = || {
        CombinedLogger::builder()
            .terminal(Config::default())
            .file(Config::default(), "/not/writable/app.log", RotationConfig::default())
            .fallback_to_terminal(true)
            .init()
    };
}
