use handsome_logger::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use log::*;
use std::fs::File;

fn main() {
    let term_config = ConfigBuilder::new()
        .set_format_text("[_msg] [_color_start][[_level]][_color_end], module [_module], line [_line]", None)
        .build();
    let write_config = ConfigBuilder::new()
        .set_format_text("[_level] [_msg] [_module]:[_line]", None)
        .set_level(LevelFilter::Trace)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(term_config, TerminalMode::Mixed, ColorChoice::Auto),
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
// Got INFO [INFO], module terminal_and_file_logging, line 21
// Got WARNING [WARN], module terminal_and_file_logging, line 22
// Got ERROR [ERROR], module terminal_and_file_logging, line 23

// Output in my_rust_binary.log
// [TRACE] Got TRACE terminal_and_file_logging:19
// [DEBUG] Got DEBUG terminal_and_file_logging:20
// [INFO] Got INFO terminal_and_file_logging:21
// [WARN] Got WARNING terminal_and_file_logging:22
// [ERROR] Got ERROR terminal_and_file_logging:23
