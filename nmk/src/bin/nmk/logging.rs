use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode};

pub fn setup(verbosity: u8) {
    let log_level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    let config = simplelog::ConfigBuilder::new()
        .set_thread_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Trace)
        .build();
    TermLogger::init(log_level, config, TerminalMode::Stderr, ColorChoice::Always)
        .expect("failed to setup logging")
}
