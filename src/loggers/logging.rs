use std::io::{Error, Write};
use std::{process, thread};

use log::Record;
use termcolor::{BufferedStandardStream, WriteColor};

use crate::config::{TimeFormat, Token};
use crate::Config;

/// Logging functionality for `WriteLogger` that can be used for any write target, even console.
/// Operate on tokens, which allow to easily change position of printing item
#[inline(always)]
pub fn try_log<W>(config: &Config, record: &Record, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    for token in &config.tokens[record.level() as usize] {
        match token {
            Token::Time => write_time(write, config, record)?,
            Token::Level => write!(write, "{}", record.level())?,
            Token::ThreadId => write_thread_id(write)?,
            Token::ThreadName => write_thread_name(write)?,
            Token::ProcessId => write!(write, "{}", process::id())?,
            Token::Module => write!(write, "{}", record.module_path().unwrap_or("<unknown>"))?,
            Token::File => write!(write, "{}", record.file().unwrap_or("<unknown>"))?,
            Token::FileName => write_file_name(record, write)?,
            Token::Line => write!(write, "{}", record.line().unwrap_or(0))?,
            Token::Text(text) => write!(write, "{text}")?,
            Token::Message => write_args(record, write)?,
            Token::ColorStart | Token::ColorEnd => {}
        }
    }
    writeln!(write)?;

    Ok(())
}

/// Terminal logging functionality, that can only be used by `TermLogger`
/// The only difference between this function and casual `try_log` is flushing at the end and
/// using tokens `ColorStart` and `ColorEnd`, that allows to write colors into terminal
#[inline(always)]
pub fn try_log_term(config: &Config, record: &Record, write: &mut BufferedStandardStream) -> Result<(), Error> {
    for token in &config.tokens[record.level() as usize] {
        match token {
            Token::Time => write_time(write, config, record)?,
            Token::Level => write!(write, "{}", record.level())?,
            Token::ThreadId => write_thread_id(write)?,
            Token::ThreadName => write_thread_name(write)?,
            Token::ProcessId => write!(write, "{}", process::id())?,
            Token::Module => write!(write, "{}", record.module_path().unwrap_or("<unknown>"))?,
            Token::File => write!(write, "{}", record.file().unwrap_or("<unknown>"))?,
            Token::FileName => write_file_name(record, write)?,
            Token::Line => write!(write, "{}", record.line().unwrap_or(0))?,
            Token::Text(text) => write!(write, "{text}")?,
            Token::Message => write_args(record, write)?,
            Token::ColorStart => set_color(write, config, record, true)?,
            Token::ColorEnd => set_color(write, config, record, false)?,
        }
    }
    writeln!(write)?;

    // The log crate holds the logger as a `static mut`, which isn't dropped
    // at program exit: https://doc.rust-lang.org/reference/items/static-items.html
    // Sadly, this means we can't rely on the BufferedStandardStreams flushing
    // themselves on the way out, so to avoid the Case of the Missing 8k,
    // flush each entry.
    write.flush()
}

/// Writes local thread id and starts from 1
#[inline(always)]
pub fn write_thread_id<W>(write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    // TODO, change this to simple `thread::current().id().as_u64()` when will be stabilized
    let thread_id_string = format!("{:?}", thread::current().id()).replace("ThreadId(", "").replace(')', "");
    write!(write, "{thread_id_string}")?;

    Ok(())
}

/// Writes thread name
#[inline(always)]
pub fn write_thread_name<W>(write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    match thread::current().name() {
        Some(thread_name) => write!(write, "{thread_name}")?,
        None => write!(write, "<unknown>")?,
    }

    Ok(())
}

/// Writes file name with its extension
/// E.g. `file.rs`
#[inline(always)]
pub fn write_file_name<W>(record: &Record, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    match record.file() {
        Some(file) => {
            let mut file = file.rsplitn(2, '/');
            if let Some(file) = file.next() {
                write!(write, "{file}")?;
            } else {
                write!(write, "<unknown>")?;
            }
        }
        None => write!(write, "<unknown>")?,
    }
    Ok(())
}

/// Writes time in choosen time format
#[inline(always)]
pub fn write_time<W>(write: &mut W, config: &Config, record: &Record) -> Result<(), Error>
where
    W: Write + Sized,
{
    use time::error::Format;
    use time::format_description::well_known::*;

    let time = time::OffsetDateTime::now_utc().to_offset(config.time_offset);
    let res = match config.time_format[record.level() as usize] {
        TimeFormat::Rfc2822 => time.format_into(write, &Rfc2822),
        TimeFormat::Rfc3339 => time.format_into(write, &Rfc3339),
        TimeFormat::Custom(format) => time.format_into(write, &format),
    };
    match res {
        Err(Format::StdIo(err)) => return Err(err),
        Err(err) => panic!("Invalid time format: {err}"),
        _ => {}
    };

    write!(write, "")?;
    Ok(())
}

/// Writes color to terminal output
#[inline(always)]
pub fn set_color(write: &mut BufferedStandardStream, config: &Config, record: &Record, color_start: bool) -> Result<(), Error> {
    if config.enabled_colors {
        if color_start {
            let color = &config.compiled_colors[record.level() as usize];
            write.set_color(color)?;
        } else {
            write.reset()?;
        }
    }

    Ok(())
}

/// Writes args provided in time macro
/// E.g. record.args() in info!("Print This") will contain one argument "Print This"
#[inline(always)]
pub fn write_args<W>(record: &Record, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    write!(write, "{}", record.args())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use log::Level;
    use termcolor::ColorChoice;

    use crate::loggers::term_logger::OutputStreams;
    use crate::ConfigBuilder;

    use super::*;

    #[test]
    fn test_tokens_output() {
        let mut config = ConfigBuilder::new().build();
        let record = Record::builder().build();
        let i = vec![
            Token::File,
            Token::Text(" "),
            Token::Level,
            Token::Text(" "),
            Token::Module,
            Token::Text(" "),
            Token::Line,
            Token::Text(" "),
            Token::FileName,
            Token::Text(" "),
            Token::ColorEnd,
            Token::Text(" "),
            Token::ColorStart,
            Token::Text(" "),
            Token::ThreadName,
            Token::Text(" "),
            Token::Message,
            Token::Text("test"),
        ];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        assert_eq!(
            String::from_utf8(res_vec).unwrap(),
            "<unknown> INFO <unknown> 0 <unknown>   loggers::logging::tests::test_tokens_output test\n".to_string()
        );

        let mut config = ConfigBuilder::new().build();
        let record = Record::builder().build();
        let i = vec![Token::ThreadId];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        let mut ret = String::from_utf8(res_vec.clone()).unwrap();
        ret.pop().unwrap();
        assert!(ret.parse::<u32>().is_ok());

        let mut config = ConfigBuilder::new().build();
        let record = Record::builder().build();
        let i = vec![Token::ProcessId];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        let mut ret = String::from_utf8(res_vec.clone()).unwrap();
        ret.pop().unwrap();
        assert!(ret.parse::<u32>().is_ok());

        let mut config = ConfigBuilder::new().build();
        let record = Record::builder().build();
        let i = vec![Token::Time];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        assert_eq!(String::from_utf8(res_vec).unwrap().len(), "20:24:46.123\n".len());
    }

    #[test]
    fn test_colours() {
        for level_filter in &[Level::Info, Level::Warn, Level::Error, Level::Debug, Level::Trace] {
            test_colour_level(*level_filter);
        }
    }

    fn test_colour_level(level: Level) {
        let mut config = ConfigBuilder::new().set_enabled_colours(true).build();
        let record = Record::builder().level(level).build();
        let i = vec![Token::Text("RAR"), Token::Level, Token::Text("RAR")];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];

        let mut streams = OutputStreams {
            err: BufferedStandardStream::stderr(ColorChoice::Always),
            out: BufferedStandardStream::stdout(ColorChoice::Always),
        };
        let res = try_log_term(&config, &record, &mut streams.err);
        assert!(res.is_ok());
    }
}
