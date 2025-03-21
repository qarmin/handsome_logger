use log::*;

fn main() {
    handsome_logger::init().unwrap();
    // or longer version which do same thing:
    // TermLogger::init(Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// 14:57:27.521 [INFO] terminal_logging: Got INFO
// 14:57:27.521 [WARN] terminal_logging: Got WARNING
// 14:57:27.521 [ERROR] terminal_logging: Got ERROR
