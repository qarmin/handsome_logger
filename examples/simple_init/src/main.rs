use log::{debug, error, info, trace, warn};

fn main() {
    handsome_logger::init().unwrap();

    trace!("Got TRACE");
    debug!("Got DEBUG");
    info!("Got INFO");
    warn!("Got WARNING");
    error!("Got ERROR");
}

// Output in terminal:
//
// 14:53:18.142 [INFO] simple_init: Got INFO
// 14:53:18.142 [WARN] simple_init: Got WARNING
// 14:53:18.142 [ERROR] simple_init: Got ERROR
