use handsome_logger::{ConfigBuilder, WriteLogger};
use log::*;
use std::fs::OpenOptions;
use std::io::{Error, Write};

fn write_formatter(record: &Record, write: &mut dyn Write) -> Result<(), Error> {
    let level = record.level();
    let level_str = match level {
        Level::Trace => "TRACERT",
        Level::Debug => "DEBUGGGER",
        Level::Info => "INFORMER",
        Level::Warn => "WARNUNGER",
        Level::Error => "ERRORER",
    };
    write!(
        write,
        "[{:10}] {}: {}\n",
        level_str,
        record.module_path().unwrap_or_default(),
        record.args()
    )?;

    Ok(())
}

fn main() {
    let write_config = ConfigBuilder::new()
        .set_level(LevelFilter::Trace)
        .set_custom_write_formatter(Some(write_formatter))
        .build();
    let write_rotater = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("write.log")
        .unwrap();

    WriteLogger::init(write_config, write_rotater).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in `write.log`:
//
// [TRACERT   ] custom_write_formatter: Got TRACE
// [DEBUGGGER ] custom_write_formatter: Got DEBUG
// [INFORMER  ] custom_write_formatter: Got INFO
// [WARNUNGER ] custom_write_formatter: Got WARNING
// [ERRORER   ] custom_write_formatter: Got ERROR
