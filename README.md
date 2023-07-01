# Handsome Logger

Handsome logger aims to be quite simple and quite powerful.

This is a fork of [simplelog.rs](https://github.com/Drakulix/simplelog.rs) from commit `70f4dcb6c20de819b68a4c52988e850403f779db` and is available under same license as the original project.

I created it because the formatting possibilities of this library were insufficient for me and the changes that would have to be made to it to "fix" it were too big.

## Features
- Colored output - at least colored is `[INFO]`, `[DEBUG]`, `[WARN]`, `[TRACE]` and `[ERROR]`
- Saving both to file and terminal
- Customizable format - place where each field is added can be easily modified
- Multiple properties - thread id, module name, file name, line number, timestamp

## Example Usage
First add to Cargo.toml, this two lines
```
handsome_logger = "0.1"
log = "0.4"
```
```
use handsome_logger::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger};
use log::{error, info,debug, trace, warn};

fn main() {
    let config = ConfigBuilder::default().set_level(handsome_logger::LevelFilter::Debug).build();

    TermLogger::init(
        config,
        handsome_logger::TerminalMode::Mixed, // Save results to stderr or stdout?
        ColorChoice::Auto,
    ).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}
```
should print
```
17:38:18 [DEBUG] [project:14] Got DEBUG
17:38:18 [INFO] [project:15] Got INFO
17:38:18 [WARN] [project:16] Got WARNING
17:38:18 [ERROR] [project:17] Got ERROR
```
## Why?
Recently I needed to use logger in my project, but I found that tested loggers not have all required features that I need:
- Formatting is simple - both env_logger and simplelog are simple to use, but for users may be a little hard to modify
- Save data in different format to file and to terminal
- Have colourful output - without it is hard to read logs

## TODO 
- Add more tests
- Set minimal required version of rust(currently only latest rust version is supported)
- Add more documentations
- Add more examples
- Add handling of default environment variables like env_logger
- Add filtering options

## License 
Apache 2.0 or MIT, at your option.

Copyright (c) 2023 Rafał Mikrut
Copyright (c) 2015-2023 Victor Brekenfeld and contributors(for full list see https://github.com/Drakulix/simplelog.rs/graphs/contributors)

