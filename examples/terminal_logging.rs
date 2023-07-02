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
// 12:47:13 [INFO] [simple_logging:9] Got INFO
// 12:47:13 [WARN] [simple_logging:10] Got WARNING
// 12:47:13 [ERROR] [simple_logging:11] Got ERROR
