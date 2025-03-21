use core::fmt::{Debug, Formatter};
use log::{LevelFilter, Record};
use std::io::{Error, Write};
use std::sync::Arc;
use termcolor::{BufferedStandardStream, Color, ColorSpec};
pub use time::format_description::FormatItem;
pub use time::macros::format_description;
pub use time::UtcOffset;

const LEVEL_NUMBER: usize = 6;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum TimeFormat {
    Rfc2822,           // Fri, 21 Nov 1997 09:55:06 -0600
    Rfc3339,           // 2015-03-15T13:45:30Z
    TimeWithMicro,     // HH:MM:SS.SSS
    DateTimeWithMicro, // YYYY-MM-DD HH:MM:SS.SSS
    Custom(&'static [FormatItem<'static>]),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Text(&'static str),
    Time,
    Level,
    ColorStart,
    ColorEnd,
    ThreadId,
    ThreadName,
    ProcessId,
    Module,
    FileName,
    File,
    Line,
    Message,
}

type FilterFunction = dyn Fn(&Record) -> bool + Send + Sync;
type WriteFunction = dyn Fn(&Record, &mut dyn Write) -> Result<(), Error> + Send + Sync;
type TerminalWriteFunction = dyn Fn(&Record, &mut BufferedStandardStream) -> Result<(), Error> + Send + Sync;

#[derive(Clone)]
pub struct Config {
    pub(crate) level: LevelFilter,
    pub(crate) time_offset: UtcOffset,
    pub(crate) write_once: bool,

    pub(crate) time_format: [TimeFormat; LEVEL_NUMBER],
    pub(crate) format_text: [&'static str; LEVEL_NUMBER],
    pub(crate) tokens: [Vec<Token>; LEVEL_NUMBER],

    // Colors
    pub(crate) colored_text_color: [Option<Color>; LEVEL_NUMBER],
    pub(crate) background_color: [Option<Color>; LEVEL_NUMBER],
    pub(crate) compiled_colors: [ColorSpec; LEVEL_NUMBER],
    pub(crate) enabled_colors: bool,

    pub(crate) message_filtering: Option<Arc<FilterFunction>>,
    pub(crate) write_formatter: Option<Arc<WriteFunction>>,
    pub(crate) terminal_formatter: Option<Arc<TerminalWriteFunction>>,
}

/// Enum representing different format texts for logging.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum FormatText {
    Simple,
    SimpleC,
    Default,
    DefaultC,
    DefaultWithFileName,
    DefaultWithFileNameC,
    DefaultWithThread,
    DefaultWithThreadC,
    DefaultWithThreadFile,
    DefaultWithThreadFileC,
}

impl FormatText {
    pub fn get(&self) -> &'static str {
        match self {
            Self::Simple => "[_time] [[_level]] [_msg]",
            Self::SimpleC => "[_time] [_color_start][[_level]][_color_end] [_msg]",
            Self::Default => "[_time] [[_level]] [_module]: [_msg]",
            Self::DefaultC => "[_time] [_color_start][[_level]][_color_end] [_module]: [_msg]",
            Self::DefaultWithFileName => "[_time] [[_level]] [[_module]] [_file_name]:[_line] - [_msg]",
            Self::DefaultWithFileNameC => "[_time] [_color_start][[_level]][_color_end] [[_module]] [_file_name]:[_line] - [_msg]",
            Self::DefaultWithThread => "[_time] [[_level]] [[_module]] ([_thread_id]) - [_msg]",
            Self::DefaultWithThreadC => "[_time] [_color_start][[_level]][_color_end] ([_thread_id]) [[_module]] - [_msg]",
            Self::DefaultWithThreadFile => "[_time] [[_level]] [[_module]] ([_thread_id]) [_file_name]:[_line] - [_msg]",
            Self::DefaultWithThreadFileC => "[_time] [_color_start][[_level]][_color_end] ([_thread_id]) [[_module]] [_file_name]:[_line] - [_msg]",
        }
    }
}

impl Config {
    /// Internal function to calculate all required data from user input
    /// this is done only once to avoid unnecessary computations
    pub(crate) fn calculate_data(&mut self) {
        self.calculate_tokens();
        self.calculate_colors();
    }

    /// Creating `ColorSpec` from user colors
    fn calculate_colors(&mut self) {
        for (idx, color_spec) in self.compiled_colors.iter_mut().enumerate() {
            *color_spec = ColorSpec::new().set_bg(self.background_color[idx]).set_fg(self.colored_text_color[idx]).clone();
        }
    }

    /// Calculate tokens from format text
    fn calculate_tokens(&mut self) {
        let allowed_tokens = [
            ("[_time]", Token::Time),
            ("[_level]", Token::Level),
            ("[_color_start]", Token::ColorStart),
            ("[_color_end]", Token::ColorEnd),
            ("[_thread_id]", Token::ThreadId),
            ("[_thread_name]", Token::ThreadName),
            ("[_process_id]", Token::ProcessId),
            ("[_module]", Token::Module),
            ("[_file]", Token::File),
            ("[_file_name]", Token::FileName),
            ("[_line]", Token::Line),
            ("[_msg]", Token::Message),
        ];

        for (idx, format_text) in self.format_text.into_iter().enumerate() {
            let mut collected_tokens = Vec::new();
            let mut current_index = 0;
            loop {
                let mut minimum_index = usize::MAX;
                let mut choose_token_text = None;
                for (token_txt, token) in &allowed_tokens {
                    if let Some(find_idx) = format_text[current_index..].find(token_txt) {
                        if minimum_index > find_idx + current_index {
                            minimum_index = find_idx + current_index;
                            choose_token_text = Some((token_txt, token));
                        }
                    }
                }

                // No more tokens
                if let Some((token_txt, token)) = choose_token_text {
                    if minimum_index > current_index {
                        let text = &format_text[current_index..minimum_index];
                        collected_tokens.push(Token::Text(text));
                    }
                    collected_tokens.push(token.clone());
                    current_index = minimum_index + token_txt.len() - 1;
                } else {
                    if current_index != format_text.len() {
                        let text = &format_text[current_index..];
                        collected_tokens.push(Token::Text(text));
                    }
                    break; // No more to check
                }
                current_index += 1;
            }

            self.tokens[idx] = collected_tokens;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self(Config::default())
    }

    /// Sets format of logged message
    /// E.g. "\[_time\] \[\[_level\]\] \[_module\] \"\[_msg\]\""
    /// depending on other settings, may print something like:
    /// 14:21:15 \[INFO\] main: "Hello world!"
    /// If level is none, it will set all levels
    pub fn set_format_text(&mut self, format_text: &'static str, level: Option<LevelFilter>) -> &mut Self {
        if let Some(level) = level {
            self.0.format_text[level as usize] = format_text;
        } else {
            self.0.format_text = [format_text; LEVEL_NUMBER];
        }
        self
    }

    /// Sets background color
    /// If color is none, background will not be colored
    /// If level is none, it will set all levels
    /// If level is some, it will set only that level
    /// Background color is used only if `enabled_colors` is true
    pub fn set_background_color(&mut self, background_color: Option<Color>, level: Option<LevelFilter>) -> &mut Self {
        if let Some(level) = level {
            self.0.background_color[level as usize] = background_color;
        } else {
            self.0.background_color = [background_color; LEVEL_NUMBER];
        }
        self
    }

    /// Sets text color
    /// If color is none, text will be invisible
    /// If level is none, it will set all levels
    /// If level is some, it will set only that level
    /// Background color is used only if `enabled_colors` is true
    pub fn set_colored_text_color(&mut self, colored_text_color: Option<Color>, level: Option<LevelFilter>) -> &mut Self {
        if let Some(level) = level {
            self.0.colored_text_color[level as usize] = colored_text_color;
        } else {
            self.0.colored_text_color = [colored_text_color; LEVEL_NUMBER];
        }
        self
    }

    /// Enables colouring of text - only works with `TermLogger`
    pub fn set_enabled_colours(&mut self, enabled_colours: bool) -> &mut Self {
        self.0.enabled_colors = enabled_colours;
        self
    }

    /// Sets the level of the logger.
    /// E.g. using `LevelFilter::Info` will print all logs with level `Info`, `Warn`, `Error`,
    /// but not `Debug` or `Trace`.
    pub fn set_level(&mut self, level: LevelFilter) -> &mut Self {
        self.0.level = level;
        self
    }

    /// Instead of writing multiple times to target, creates a buffer, writes to memory and
    /// at the end writes only once to target
    /// This is useful when saving to file, because allows to not split one log into multiple
    /// files if rotating is used.
    /// Works only with `WriteLogger`
    pub fn set_write_once(&mut self, write_once: bool) -> &mut Self {
        self.0.write_once = write_once;
        self
    }

    /// Set time format used in logger
    /// If level is none, it will set all levels
    /// Time format can be predefined(Rfc2822 or Rfc3339) or custom
    pub fn set_time_format(&mut self, time_format: TimeFormat, level: Option<LevelFilter>) -> &mut Self {
        if let Some(level) = level {
            self.0.time_format[level as usize] = time_format;
        } else {
            self.0.time_format = [time_format; LEVEL_NUMBER];
        }
        self
    }

    /// Manually sets the offset used for the time.
    pub fn set_time_offset(&mut self, offset: UtcOffset) -> &mut Self {
        self.0.time_offset = offset;
        self
    }

    /// Sets the offset used to the current local time offset
    pub fn set_time_offset_to_local(&mut self) -> Result<&mut Self, &mut Self> {
        match Self::get_local_time_offset() {
            Some(offset) => {
                self.0.time_offset = offset;
                Ok(self)
            }
            None => Err(self),
        }
    }

    /// Reset the offset used to UTC
    pub fn set_remove_time_offset(&mut self) -> &mut Self {
        self.0.time_offset = UtcOffset::UTC;
        self
    }

    /// Sets function that will be used to filter messages
    /// If function returns true, message will be logged, otherwise it will be ignored
    /// Function takes as argument function that will be filtered allowed results
    /// If `message_filtering` is none, all messages will be logged
    /// ```
    /// use log::{info, Record};
    /// use handsome_logger::{Config, ConfigBuilder};
    ///
    /// fn filtering_messages(record: &Record) -> bool {
    ///     if let Some(arg) = record.args().as_str() {
    ///         !arg.contains("E")
    ///     } else {
    ///         true
    ///     }
    /// }
    ///
    /// let logger = ConfigBuilder::new().set_message_filtering(Some(filtering_messages)).build();
    /// info!("Got BED"); // This will be ignored
    /// info!("Got ANANAS"); // This will be printed
    /// ```
    pub fn set_message_filtering<F>(&mut self, message_filtering: Option<F>) -> &mut Self
    where
        F: Fn(&Record) -> bool + Send + Sync + 'static,
    {
        if let Some(message_filtering) = message_filtering {
            self.0.message_filtering = Some(Arc::new(message_filtering));
        } else {
            self.0.message_filtering = None;
        }
        self
    }

    /// Sets custom formatter for `WriteLogger`
    /// If you don't want to use default formatter, you can set your own
    /// Setting `write_formatter` to None will use default formatter
    /// Function takes as argument function that will be filtered allowed results
    pub fn set_custom_write_formatter<F>(&mut self, write_formatter: Option<F>) -> &mut Self
    where
        F: Fn(&Record, &mut dyn Write) -> Result<(), Error> + Send + Sync + 'static,
    {
        if let Some(write_formatter) = write_formatter {
            self.0.write_formatter = Some(Arc::new(write_formatter));
        } else {
            self.0.write_formatter = None;
        }
        self
    }

    /// Sets custom formatter for `TermLogger`
    /// If you don't want to use default formatter, you can set your own
    /// Setting `terminal_formatter` to None will use default formatter
    /// Function takes as argument function that will be filtered allowed results
    pub fn set_custom_terminal_formatter<F>(&mut self, terminal_formatter: Option<F>) -> &mut Self
    where
        F: Fn(&Record, &mut BufferedStandardStream) -> Result<(), Error> + Send + Sync + 'static,
    {
        if let Some(terminal_formatter) = terminal_formatter {
            self.0.terminal_formatter = Some(Arc::new(terminal_formatter));
        } else {
            self.0.terminal_formatter = None;
        }
        self
    }

    /// Gets the local time offset
    /// On unix will this use tz-rs crate,
    /// otherwise it will use time crate
    fn get_local_time_offset() -> Option<UtcOffset> {
        #[cfg(target_family = "unix")]
        {
            let Ok(timezone) = tz::TimeZone::local() else {
                return None;
            };
            let Ok(time_type) = timezone.find_current_local_time_type() else {
                return None;
            };
            UtcOffset::from_whole_seconds(time_type.ut_offset()).ok()
        }
        #[cfg(not(target_family = "unix"))]
        {
            time::UtcOffset::current_local_offset().ok()
        }
    }

    /// Builds the config
    pub fn build(&mut self) -> Config {
        self.0.clone()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        let tz_offset = ConfigBuilder::get_local_time_offset().unwrap_or(UtcOffset::UTC);

        Self {
            level: LevelFilter::Info,
            write_once: false,
            time_format: [TimeFormat::TimeWithMicro; LEVEL_NUMBER],
            time_offset: tz_offset,

            tokens: [vec![], vec![], vec![], vec![], vec![], vec![]],
            colored_text_color: [
                None,
                Some(Color::Red),    // Error
                Some(Color::Yellow), // Warn
                Some(Color::Blue),   // Info
                Some(Color::Cyan),   // Debug
                Some(Color::White),  // Trace
            ],

            background_color: [None, None, None, None, None, None],
            enabled_colors: true,
            format_text: [FormatText::DefaultC.get(); LEVEL_NUMBER],
            compiled_colors: [ColorSpec::new(), ColorSpec::new(), ColorSpec::new(), ColorSpec::new(), ColorSpec::new(), ColorSpec::new()],
            message_filtering: None,
            write_formatter: None,
            terminal_formatter: None,
        }
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("level", &self.level)
            .field("write_once", &self.write_once)
            .field("time_format", &self.time_format)
            .field("time_offset", &self.time_offset)
            .field("tokens", &self.tokens)
            .field("colored_text_color", &self.colored_text_color)
            .field("background_color", &self.background_color)
            .field("enabled_colors", &self.enabled_colors)
            .field("format_text", &self.format_text)
            .field("compiled_colors", &self.compiled_colors)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let text = "[_time] [_level] [_thread_id] [_thread_name] [_process_id] [_module] [_file][_file_name] [_line] [_color_start][_msg][_color_end] [RAR]";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(
            config.tokens[0],
            vec![
                Token::Time,
                Token::Text(" "),
                Token::Level,
                Token::Text(" "),
                Token::ThreadId,
                Token::Text(" "),
                Token::ThreadName,
                Token::Text(" "),
                Token::ProcessId,
                Token::Text(" "),
                Token::Module,
                Token::Text(" "),
                Token::File,
                Token::FileName,
                Token::Text(" "),
                Token::Line,
                Token::Text(" "),
                Token::ColorStart,
                Token::Message,
                Token::ColorEnd,
                Token::Text(" [RAR]"),
            ]
        );

        let text = "]]][[";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(config.tokens[0], vec![Token::Text("]]][[")]);

        let text = " [_time]";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(config.tokens[0], vec![Token::Text(" "), Token::Time]);

        let text = "[_time]";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(config.tokens[0], vec![Token::Time]);

        let text = "[_time] ";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(config.tokens[0], vec![Token::Time, Token::Text(" ")]);

        let text = "";
        let mut config = ConfigBuilder::new().set_format_text(text, None).build();
        config.calculate_data();
        assert_eq!(config.tokens[0], vec![]);
    }
}
