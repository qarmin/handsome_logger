use log::LevelFilter;
use termcolor::Color;
pub use time::format_description::FormatItem;
pub use time::macros::format_description;
pub use time::UtcOffset;

#[derive(Debug, Clone, Copy)]
pub enum TimeFormat {
    Rfc2822,
    Rfc3339,
    Custom(&'static [FormatItem<'static>]),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Text(&'static str),
    Time,
    Level,
    Thread,
    Module,
    File,
    Line,
    Message,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) level: LevelFilter,
    pub(crate) time_offset: UtcOffset,

    pub(crate) time_format: [TimeFormat; 6],
    pub(crate) format_text: [&'static str; 6],
    pub(crate) tokens: [Vec<Token>; 6],

    pub(crate) level_color: [Option<Color>; 6],
    pub(crate) enabled_colors: bool,
}

const DEFAULT_FORMAT_TEXT: &str = "[_time] [_level] [[_module]:[_line]] [_msg]";

impl Config {
    /// Calculate tokens from format text
    pub(crate) fn calculate_tokens(&mut self) {
        let allowed_tokens = [
            ("[_time]", Token::Time),
            ("[_level]", Token::Level),
            ("[_thread]", Token::Thread),
            ("[_module]", Token::Module),
            ("[_file]", Token::File),
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
    pub fn new() -> ConfigBuilder {
        ConfigBuilder(Config::default())
    }

    pub fn set_format_text(&mut self, format_text: &'static str, level: Option<LevelFilter>) -> &mut ConfigBuilder {
        if let Some(level) = level {
            self.0.format_text[level as usize] = format_text;
        } else {
            self.0.format_text = [format_text; 6];
        }
        self
    }

    pub fn set_enabled_colours(&mut self, enabled_colours: bool) -> &mut ConfigBuilder {
        self.0.enabled_colors = enabled_colours;
        self
    }

    pub fn set_level(&mut self, level: LevelFilter) -> &mut ConfigBuilder {
        self.0.level = level;
        self
    }

    pub fn set_time_format(&mut self, time_format: TimeFormat, level: Option<LevelFilter>) -> &mut ConfigBuilder {
        if let Some(level) = level {
            self.0.time_format[level as usize] = time_format;
        } else {
            self.0.time_format = [time_format; 6];
        }
        self
    }

    pub fn set_time_offset(&mut self, offset: UtcOffset) -> &mut ConfigBuilder {
        self.0.time_offset = offset;
        self
    }

    /// Sets the offset used to the current local time offset
    /// (overriding values previously set by [`ConfigBuilder::set_time_offset`]).
    ///
    /// This function may fail if the offset cannot be determined soundly.
    /// This may be the case, when the program is multi-threaded by the time of calling this function.
    /// One can opt-out of this behavior by setting `RUSTFLAGS="--cfg unsound_local_offset"`.
    /// Doing so is not recommended, completely untested and may cause unexpected segfaults.
    #[cfg(feature = "time-local-offset")]
    pub fn set_time_offset_to_local(&mut self) -> Result<&mut ConfigBuilder, &mut ConfigBuilder> {
        match UtcOffset::current_local_offset() {
            Ok(offset) => {
                self.0.time_offset = offset;
                Ok(self)
            }
            Err(_) => Err(self),
        }
    }

    /// Sets the offset used to the current local time offset
    /// It is quite unsound so may cause crashes
    #[cfg(feature = "chrono-local-offset")]
    pub fn set_chrono_local_time_offset(&mut self) -> Result<&mut ConfigBuilder, &mut ConfigBuilder> {
        match UtcOffset::from_whole_seconds(chrono::offset::Local::now().offset().local_minus_utc()) {
            Ok(offset) => {
                self.0.time_offset = offset;
                Ok(self)
            }
            Err(_) => Err(self),
        }
    }

    pub fn build(&mut self) -> Config {
        let mut cloned_config = self.0.clone();
        cloned_config.calculate_tokens();
        cloned_config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder::new()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            level: LevelFilter::Info,
            time_format: [TimeFormat::Custom(format_description!("[hour]:[minute]:[second]")); 6],
            time_offset: UtcOffset::UTC,

            tokens: [vec![], vec![], vec![], vec![], vec![], vec![]],
            level_color: [
                None,                // Default foreground
                Some(Color::Red),    // Error
                Some(Color::Yellow), // Warn
                Some(Color::Blue),   // Info
                Some(Color::Cyan),   // Debug
                Some(Color::White),  // Trace
            ],

            enabled_colors: true,
            format_text: [DEFAULT_FORMAT_TEXT; 6],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let text = "[_time] [_level] [_thread] [_module] [_file] [_line] [_msg] [RAR]";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(
            config.tokens[0],
            vec![
                Token::Time,
                Token::Text(" "),
                Token::Level,
                Token::Text(" "),
                Token::Thread,
                Token::Text(" "),
                Token::Module,
                Token::Text(" "),
                Token::File,
                Token::Text(" "),
                Token::Line,
                Token::Text(" "),
                Token::Message,
                Token::Text(" [RAR]"),
            ]
        );

        let text = "]]][[";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(config.tokens[0], vec![Token::Text("]]][[")]);

        let text = " [_time]";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(config.tokens[0], vec![Token::Text(" "), Token::Time]);

        let text = "[_time]";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(config.tokens[0], vec![Token::Time]);

        let text = "[_time] ";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(config.tokens[0], vec![Token::Time, Token::Text(" ")]);

        let text = "";
        let config = ConfigBuilder::new().set_format_text(text, None).build();
        assert_eq!(config.tokens[0], vec![]);
    }
}
