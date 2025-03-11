use handsome_logger::{CombinedLogger, ConfigBuilder, FormatText, TermLogger, WriteLogger};
use log::*;
use std::fs::File;

fn main() {
    let term_config = ConfigBuilder::new()
        .set_format_text(FormatText::DefaultWithFileName.get(), None)
        .build();
    let write_config = ConfigBuilder::new()
        .set_format_text("[[_level]] \"[_msg]\" [[_module]] - [_file_name]:[_line]", None)
        .set_level(LevelFilter::Trace)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new_from_config(term_config),
        WriteLogger::new(write_config, File::create("my_rust_binary.log").unwrap()),
    ])
    .unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// Got INFO [INFO], module terminal_and_file_logging, line 22
// Got WARNING [WARN], module terminal_and_file_logging, line 23
// Got ERROR [ERROR], module terminal_and_file_logging, line 24

// Output in my_rust_binary.log

// [TRACE] "Got TRACE" [terminal_and_file_logging] - main.rs:23
// [DEBUG] "Got DEBUG" [terminal_and_file_logging] - main.rs:24
// [INFO] "Got INFO" [terminal_and_file_logging] - main.rs:25
// [WARN] "Got WARNING" [terminal_and_file_logging] - main.rs:26
// [ERROR] "Got ERROR" [terminal_and_file_logging] - main.rs:27
