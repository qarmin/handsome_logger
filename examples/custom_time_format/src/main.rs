use handsome_logger::{format_description, ColorChoice, ConfigBuilder, TermLogger, TerminalMode, TimeFormat};
use log::*;

fn main() {
    let mut term_config_builder = ConfigBuilder::new();

    let term_config = term_config_builder
        .set_time_format(
            TimeFormat::Custom(format_description!(
                "[year]::[month]::[day]  [hour]__[minute]__[second]"
            )),
            None,
        )
        .set_remove_time_offset() // Uses UTC time instead of local time
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
// 2023::07::04  20__31__25 [INFO] custom_time_format: Got INFO
// 2023::07::04  20__31__25 [WARN] custom_time_format: Got WARNING
// 2023::07::04  20__31__25 [ERROR] custom_time_format: Got ERROR
