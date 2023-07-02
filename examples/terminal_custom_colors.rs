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
        .set_foreground_color(Some(Color::Black), None)
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
// 15:18:18 [TRACE] [terminal_custom_colors:17] Got TRACE
// 15:18:18 [DEBUG] [terminal_custom_colors:18] Got DEBUG
// 15:18:18 [INFO] [terminal_custom_colors:19] Got INFO
// 15:18:18 [WARN] [terminal_custom_colors:20] Got WARNING
// 15:18:18 [ERROR] [terminal_custom_colors:21] Got ERROR
