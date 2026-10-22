use handsome_logger::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use log::{LevelFilter, info};

mod network {
    pub fn connect() {
        log::info!("Connected");
    }

    pub mod noisy {
        pub fn ping() {
            log::info!("Ping");
        }
    }
}

fn main() {
    // Only this crate is logged, `network::noisy` is skipped
    let config = ConfigBuilder::new()
        .set_level(LevelFilter::Trace)
        .set_allowed_modules(&["filter_modules"])
        .set_ignored_modules(&["filter_modules::network::noisy"])
        .build();
    TermLogger::init(config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    info!("Got INFO");
    network::connect();
    network::noisy::ping(); // Not printed
}
