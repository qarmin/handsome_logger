use handsome_logger::{Color, ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use log::*;

fn main() {
    let term_config = ConfigBuilder::new()
        .set_background_color(Some(Color::Green), Some(LevelFilter::Trace))
        .set_colored_text_color(Some(Color::Rgb(255, 255, 255)), Some(LevelFilter::Trace))
        .set_format_text("[[_level]] [_color_start][_msg][_color_end]", None)
        .set_format_text("[[_level]] [_color_start][_msg][_color_end] [_module]", Some(LevelFilter::Debug))
        .set_format_text("[_color_start][[_level]] [_msg] [_module]:[_line][_color_end]", Some(LevelFilter::Error))
        .set_format_text("[[_level]] [_msg] [_module]:[_line] [_color_start][_file][_color_end]", Some(LevelFilter::Trace))
        .set_level(LevelFilter::Trace)
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
// [TRACE] Got TRACE custom_formatting_in_each_level:15 examples/custom_formatting_in_each_level.rs
// [DEBUG] Got DEBUG custom_formatting_in_each_level
// [INFO] Got INFO
// [WARN] Got WARNING
// [ERROR] Got ERROR custom_formatting_in_each_level:19
