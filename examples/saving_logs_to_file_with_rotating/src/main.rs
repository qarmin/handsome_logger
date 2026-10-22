use handsome_logger::{ConfigBuilder, Naming, RotatingFile, RotationConfig, RotationPeriod, RotationType, WriteLogger};
use log::*;

fn main() {
    let write_config = ConfigBuilder::new()
        .set_format_text("[[_level]] \"[_msg]\" [[_module]] - [_file_name]:[_line]", None)
        .set_level(LevelFilter::Trace)
        .build();

    // Rotate when file would exceed 200 bytes or when the day changes, keep 5 rotated files
    let rotation = RotationConfig::new(RotationType::TimeSize {
        period: RotationPeriod::Daily,
        max_bytes: 200,
    })
    .naming(Naming::Timestamp)
    .max_files(5);

    let rotating_file = RotatingFile::new("my_rust_binary.log", rotation).unwrap();

    // init_rotating makes sure that one message is never split between two files
    WriteLogger::init_rotating(write_config, rotating_file).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// 2 files will be created with such content:
// - my_rust_binary.log
// [WARN] "Got WARNING" [saving_logs_to_file_with_rotating] - main.rs:31
// [ERROR] "Got ERROR" [saving_logs_to_file_with_rotating] - main.rs:32
//
// - my_rust_binary.log.20230704T220727 - rotated file
// [TRACE] "Got TRACE" [saving_logs_to_file_with_rotating] - main.rs:28
// [DEBUG] "Got DEBUG" [saving_logs_to_file_with_rotating] - main.rs:29
// [INFO] "Got INFO" [saving_logs_to_file_with_rotating] - main.rs:30
