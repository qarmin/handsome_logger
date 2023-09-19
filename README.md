# Handsome Logger

Handsome logger aims to be fast, easy to use and configurable logger.

This is a fork of [simplelog.rs](https://github.com/Drakulix/simplelog.rs) from commit `70f4dcb6c20de819b68a4c52988e850403f779db` and is available under same license as the original project.

I created it because the formatting abilities of this library were insufficient for me and the changes that would have to be made to it to "fix" it were too big.

![Example](https://github.com/qarmin/handsome_logger/assets/41945903/f409c771-abb5-47dd-acfe-0aa385475591)

## Features
- Multiple loggers - SimpleLogger(simplest and the stablest), TermLogger(SimpleLogger + colored output), WriteLogger(can save logs e.g. to file), CombinedLogger(can combine multiple loggers and save logs, both to file and to terminal)
- Customizable format - each element, like timestamp or module name, log level, can be customized
- Colored output - you can colorize any part of log message
- Simple to use - library can be easily configured in few lines of code for most use cases
- Ability to set log level by environment variable e.g. `RUST_LOG=error ./app`
- Filtering messages - you can ignore any message basing on your own function
- Multiple log message formatters(you can use them more than once - nobody can stop you):
```
[_line] - prints line of code where log was called or 0 if cannot read line
[_file] - prints full project path to file where log was called if is inside repository of full path if is outside, or "<unknown>" if cannot read file path
[_file_name] - prints file name where log was called or "<unknown>" if cannot read file name
[_module] - prints module name where log was called or "<unknown>" if cannot read module name
[_msg] - prints user log message
[_time] - prints time of logged message
[_level] - prints log level (INFO, DEBUG, etc.)
[_thread_id] - prints thread id
[_thread_name] - prints thread name
[_process_id] - prints process id
[_color_start], [_color_end] - starts and ends colorization of log message
```
## Example Usage
First add to Cargo.toml, this two lines
```
handsome_logger = "0.3"
log = "0.4"
```
```rust
use handsome_logger::{ColorChoice, Config, TermLogger, TerminalMode};
use log::*;

fn main() {
    TermLogger::init(Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}
```
should print
```
21:20:22 [INFO] terminal_logging: Got INFO
21:20:22 [WARN] terminal_logging: Got WARNING
21:20:22 [ERROR] terminal_logging: Got ERROR
```

examples folder contains examples of
- formatting logs
- saving logs to file and rotating it
- using multiple loggers
- colouring terminal output
- filtering messages

## License 
Apache 2.0 or MIT, at your option.

Copyright (c) 2023 Rafał Mikrut  
Copyright (c) 2015-2023 Victor Brekenfeld and contributors(for full list see https://github.com/Drakulix/simplelog.rs/graphs/contributors)

