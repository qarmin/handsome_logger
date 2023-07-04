use handsome_logger::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use log::{debug, error, info, trace, warn};

fn main() {
    let config_preset = ConfigBuilder::new_preset_config_full().build();
    TermLogger::init(config_preset, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// 2023-07-04 18:41:24.451 +00 [INFO] [terminal_logging_preset] main.rs:10 - Got INFO
// 2023-07-04 18:41:24.451 +00 [WARN] [terminal_logging_preset] main.rs:11 - Got WARNING
// 2023-07-04 18:41:24.451 +00 [ERROR] [terminal_logging_preset] main.rs:12 - Got ERROR
