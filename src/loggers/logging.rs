use std::io::{Error, Write};
use std::thread;

use log::Record;
use termcolor::{BufferedStandardStream, ColorSpec, WriteColor};

use crate::config::{TimeFormat, Token};
use crate::Config;

#[inline(always)]
pub fn try_log<W>(config: &Config, record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    for token in &config.tokens[record.level() as usize] {
        match token {
            Token::Time => write_time(write, config, record)?,
            Token::Level => write!(write, "[{}]", record.level())?,
            Token::Thread => write!(write, "{:?}", thread::current().id())?,
            Token::Module => write!(write, "{}", record.module_path().unwrap_or("<unknown>"))?,
            Token::File => write!(write, "{}", record.file().unwrap_or("<unknown>"))?,
            Token::Line => write!(write, "{}", record.line().unwrap_or(0))?,
            Token::Text(text) => write!(write, "{text}")?,
            Token::Message => write_args(record, write)?,
        }
    }
    writeln!(write)?;

    Ok(())
}

#[inline(always)]
pub fn try_log_term(config: &Config, record: &Record<'_>, write: &mut BufferedStandardStream) -> Result<(), Error> {
    for token in &config.tokens[record.level() as usize] {
        match token {
            Token::Time => write_time(write, config, record)?,
            Token::Level => write_level(write, config, record)?,
            Token::Thread => write!(write, "{:?}", thread::current().id())?,
            Token::Module => write!(write, "{}", record.module_path().unwrap_or("<unknown>"))?,
            Token::File => write!(write, "{}", record.file().unwrap_or("<unknown>"))?,
            Token::Line => write!(write, "{}", record.line().unwrap_or(0))?,
            Token::Text(text) => write!(write, "{text}")?,
            Token::Message => write_args(record, write)?,
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

#[inline(always)]
pub fn write_time<W>(write: &mut W, config: &Config, record: &Record<'_>) -> Result<(), Error>
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

#[inline(always)]
pub fn write_level(write: &mut BufferedStandardStream, config: &Config, record: &Record<'_>) -> Result<(), Error> {
    let color = config.level_color[record.level() as usize];
    if config.enabled_colors {
        write.set_color(ColorSpec::new().set_fg(color))?;
    }

    let level = format!("[{}]", record.level());

    write!(write, "{level}")?;

    if config.enabled_colors {
        write.reset()?;
    }

    Ok(())
}

#[inline(always)]
pub fn write_args<W>(record: &Record<'_>, write: &mut W) -> Result<(), Error>
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
        let i = vec![Token::File, Token::Level, Token::Module, Token::Line, Token::Text("test")];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        assert_eq!(String::from_utf8(res_vec).unwrap(), "<unknown>[INFO]<unknown>0test\n".to_string());

        let mut config = ConfigBuilder::new().build();
        let record = Record::builder().build();
        let i = vec![Token::Time];
        config.tokens = [i.clone(), i.clone(), i.clone(), i.clone(), i.clone(), i.clone()];
        let mut res_vec = Vec::new();
        let res = try_log(&config, &record, &mut res_vec);
        assert!(res.is_ok());
        assert_eq!(String::from_utf8(res_vec).unwrap().len(), "20:24:46\n".len());
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
        let res = try_log_term(&config, &record, &mut streams.out);
        assert!(res.is_ok());
    }
}
