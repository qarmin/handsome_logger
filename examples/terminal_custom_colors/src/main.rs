use handsome_logger::{Color, ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use log::*;

fn main() {
    let config = ConfigBuilder::default()
        .set_level(LevelFilter::Trace)
        .set_background_color(Some(Color::White), None) // First set white color for all levels, and then override it for each level
        .set_background_color(Some(Color::Red), Some(LevelFilter::Trace))
        .set_background_color(Some(Color::Green), Some(LevelFilter::Debug))
        .set_background_color(Some(Color::Cyan), Some(LevelFilter::Info))
        .set_background_color(Some(Color::Rgb(115, 172, 22)), Some(LevelFilter::Warn))
        .set_colored_text_color(Some(Color::Black), None)
        .build();

    TermLogger::init(config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal, with colored background of level name:
//
// 14:57:06.908 [TRACE] terminal_custom_colors: Got TRACE
// 14:57:06.908 [DEBUG] terminal_custom_colors: Got DEBUG
// 14:57:06.908 [INFO] terminal_custom_colors: Got INFO
// 14:57:06.908 [WARN] terminal_custom_colors: Got WARNING
// 14:57:06.908 [ERROR] terminal_custom_colors: Got ERROR
