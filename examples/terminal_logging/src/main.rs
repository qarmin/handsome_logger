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

// Output in terminal:
//
// 18:41:01 [INFO] terminal_logging: Got INFO
// 18:41:01 [WARN] terminal_logging: Got WARNING
// 18:41:01 [ERROR] terminal_logging: Got ERROR
