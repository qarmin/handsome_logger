use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

use log::SetLoggerError;
use termcolor::ColorChoice;

use crate::{CombinedLogger, Config, RotatingFile, RotationConfig, SharedLogger, TermLogger, TerminalMode, WriteLogger};

#[derive(Debug)]
pub enum InitError {
    Io(io::Error),
    SetLogger(SetLoggerError),
    NoOutputs,
}

impl Display for InitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "cannot open log file: {error}"),
            Self::SetLogger(error) => write!(f, "cannot set logger: {error}"),
            Self::NoOutputs => write!(f, "neither terminal nor file output was configured"),
        }
    }
}

impl std::error::Error for InitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::SetLogger(error) => Some(error),
            Self::NoOutputs => None,
        }
    }
}

impl From<io::Error> for InitError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SetLoggerError> for InitError {
    fn from(error: SetLoggerError) -> Self {
        Self::SetLogger(error)
    }
}

/// Outputs that were really enabled by `CombinedLoggerBuilder::init`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveOutputs {
    Terminal,
    File,
    TerminalAndFile,
}

struct TerminalSetup {
    config: Config,
    mode: TerminalMode,
    color_choice: ColorChoice,
}

struct FileSetup {
    config: Config,
    path: PathBuf,
    rotation: RotationConfig,
}

#[derive(Default)]
pub struct CombinedLoggerBuilder {
    terminal: Option<TerminalSetup>,
    file: Option<FileSetup>,
    fallback_to_terminal: bool,
}

impl CombinedLoggerBuilder {
    /// Terminal output with `TerminalMode::Mixed` and `ColorChoice::Auto`
    #[must_use]
    pub fn terminal(self, config: Config) -> Self {
        self.terminal_with(config, TerminalMode::Mixed, ColorChoice::Auto)
    }

    #[must_use]
    pub fn terminal_with(mut self, config: Config, mode: TerminalMode, color_choice: ColorChoice) -> Self {
        self.terminal = Some(TerminalSetup { config, mode, color_choice });
        self
    }

    #[must_use]
    pub fn file(mut self, config: Config, path: impl Into<PathBuf>, rotation: RotationConfig) -> Self {
        self.file = Some(FileSetup {
            config,
            path: path.into(),
            rotation,
        });
        self
    }

    /// When the log file cannot be opened, logs to terminal only (using default terminal settings if none were set) instead of returning error
    #[must_use]
    pub const fn fallback_to_terminal(mut self, fallback_to_terminal: bool) -> Self {
        self.fallback_to_terminal = fallback_to_terminal;
        self
    }

    pub fn init(self) -> Result<ActiveOutputs, InitError> {
        if self.terminal.is_none() && self.file.is_none() {
            return Err(InitError::NoOutputs);
        }

        let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
        let mut terminal = self.terminal;
        let mut has_file = false;

        if let Some(file) = self.file {
            match RotatingFile::new(file.path, file.rotation) {
                Ok(rotating) => {
                    loggers.push(WriteLogger::new_rotating(file.config, rotating));
                    has_file = true;
                }
                Err(_) if self.fallback_to_terminal => {
                    terminal.get_or_insert_with(|| TerminalSetup {
                        config: Config::default(),
                        mode: TerminalMode::Mixed,
                        color_choice: ColorChoice::Auto,
                    });
                }
                Err(error) => return Err(error.into()),
            }
        }

        let has_terminal = terminal.is_some();
        if let Some(terminal) = terminal {
            loggers.insert(0, TermLogger::new(terminal.config, terminal.mode, terminal.color_choice));
        }

        CombinedLogger::init(loggers)?;
        Ok(match (has_terminal, has_file) {
            (true, true) => ActiveOutputs::TerminalAndFile,
            (true, false) => ActiveOutputs::Terminal,
            _ => ActiveOutputs::File,
        })
    }
}
