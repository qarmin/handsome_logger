use handsome_logger::{format_description, ColorChoice, ConfigBuilder, TermLogger, TerminalMode, TimeFormat};
use log::*;

fn main() {
    let mut term_config_builder = ConfigBuilder::new();
    #[cfg(feature = "time")]
    let term_config = term_config_builder
        .set_time_offset_to_local() // This may not work with multithreaded app, consider to check set_chrono_local_time_offset
        .unwrap();
    let term_config = term_config_builder
        .set_time_format(TimeFormat::Custom(format_description!("[year]::[month]::[day]  [hour]__[minute]__[second]")), None)
        .build();

    TermLogger::init(term_config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// 2023::07::02  15__11__15 [INFO] [custom_time_format:16] Got INFO
// 2023::07::02  15__11__15 [WARN] [custom_time_format:17] Got WARNING
// 2023::07::02  15__11__15 [ERROR] [custom_time_format:18] Got ERROR
