use handsome_logger::{ColorChoice, ConfigBuilder, FormatText, TermLogger, TerminalMode};
use log::{debug, error, info, trace, warn, LevelFilter, Record};

fn filtering_messages(record: &Record) -> bool {
    if let Some(arg) = record.args().as_str() {
        !arg.contains("E")
    } else {
        true
    }
}

fn main() {
    let config_preset = ConfigBuilder::new()
        .set_format_text(FormatText::DefaultWithFileName.get(), None)
        .set_level(LevelFilter::Trace)
        .set_message_filtering(Some(filtering_messages))
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
// 2023-07-08 10:12:40.220 +00 [INFO] [filter_messages] main.rs:21 - Got INFO
// 2023-07-08 10:12:40.220 +00 [WARN] [filter_messages] main.rs:22 - Got WARNING
