use handsome_logger::{ColorChoice, Config, ConfigBuilder, TermLogger, TerminalMode};
use log::*;

fn write_formatter(record: &Record, write: &mut Write) -> bool {
    if let Some(arg) = record.args().as_str() {
        !arg.contains("E")
    } else {
        true
    }
}

fn main() {
    let config_preset = ConfigBuilder::new()
        .set_level(LevelFilter::Trace)
        .set_custom_write_formatter(Some(filtering_messages))
        .build();
    TermLogger::init(config_preset, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// 18:41:01 [INFO] terminal_logging: Got INFO
// 18:41:01 [WARN] terminal_logging: Got WARNING
// 18:41:01 [ERROR] terminal_logging: Got ERROR
