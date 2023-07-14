use handsome_logger::{Color, ColorChoice, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use log::*;
use std::fs::OpenOptions;
use std::io::{Error, Write};
use termcolor::{BufferedStandardStream, ColorSpec, WriteColor};

fn terminal_formatter(record: &Record, buf_stream: &mut BufferedStandardStream) -> Result<(), Error> {
    let level_str = match record.level() {
        Level::Trace => "TRACERT",
        Level::Debug => "DEBUGGGER",
        Level::Info => "INFORMER",
        Level::Warn => "WARNUNGER",
        Level::Error => "ERRORER",
    };
    buf_stream.set_color(
        ColorSpec::new()
            .set_bg(Some(Color::Black))
            .set_fg(Some(Color::Cyan))
            .set_bold(true),
    )?;
    write!(
        buf_stream,
        "[{:10}] {}: {}\n",
        level_str,
        record.module_path().unwrap_or_default(),
        record.args()
    )?;
    buf_stream.reset()?;
    buf_stream.flush()?;
    Ok(())
}

fn main() {
    let term_config = ConfigBuilder::new()
        .set_level(LevelFilter::Debug)
        .set_custom_terminal_formatter(Some(terminal_formatter))
        .build();

    TermLogger::init(term_config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Should print in terminal bold blue text on black background:
//
// [DEBUGGGER ] custom_terminal_formatter: Got DEBUG
// [INFORMER  ] custom_terminal_formatter: Got INFO
// [WARNUNGER ] custom_terminal_formatter: Got WARNING
// [ERRORER   ] custom_terminal_formatter: Got ERROR
