use file_rotate::compression::Compression;
use file_rotate::suffix::{AppendTimestamp, FileLimit};
use file_rotate::{ContentLimit, FileRotate};
use handsome_logger::{ConfigBuilder, WriteLogger};
use log::*;

fn main() {
    let write_config = ConfigBuilder::new()
        .set_format_text("[[_level]] \"[_msg]\" [[_module]] - [_file_name]:[_line]", None)
        .set_write_once(true) // Use this when rotating files
        .set_level(LevelFilter::Trace)
        .build();

    let write_rotater = FileRotate::new(
        "my_rust_binary.log",
        AppendTimestamp::default(FileLimit::MaxFiles(10)),
        ContentLimit::BytesSurpassed(200),
        Compression::None,
        None,
    );

    WriteLogger::init(write_config, write_rotater).unwrap();
    for _ in 0..1 {
        trace!("Got TRACE");
        debug!("Got DEBUG");
        info!("Got INFO");
        warn!("Got WARNING");
        error!("Got ERROR");
    }
}

// With set_write_once set at false(default value), 2 files will be created with such content:
// - my_rust_binary.log
// :26
// [WARN] "Got WARNING" [saving_logs_to_file_with_rotating] - main.rs:27
// [ERROR] "Got ERROR" [saving_logs_to_file_with_rotating] - main.rs:28
//
// - my_rust_binary.log.20230704T220727 - rotated file
// [TRACE] "Got TRACE" [saving_logs_to_file_with_rotating] - main.rs:24
// [DEBUG] "Got DEBUG" [saving_logs_to_file_with_rotating] - main.rs:25
// [INFO] "Got INFO" [saving_logs_to_file_with_rotating] - main.rs

// As you can see, one info!() message is split between 2 files. Because ContentLimit::BytesSurpassed
// split content when write occurs. To avoid this, set set_write_once to true, which will write
// content to the file only once per message.

// With set_write_once set at true, 2 files will be created with such content:
// - my_rust_binary.log
// [WARN] "Got WARNING" [saving_logs_to_file_with_rotating] - main.rs:27
// [ERROR] "Got ERROR" [saving_logs_to_file_with_rotating] - main.rs:28
//
// - my_rust_binary.log.20230704T220727 - rotated file
// [TRACE] "Got TRACE" [saving_logs_to_file_with_rotating] - main.rs:24
// [DEBUG] "Got DEBUG" [saving_logs_to_file_with_rotating] - main.rs:25
// [INFO] "Got INFO" [saving_logs_to_file_with_rotating] - main.rs:26
