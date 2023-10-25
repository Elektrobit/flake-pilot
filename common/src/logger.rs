use chrono::Local;
use colored::{self, Colorize};
use log::{Level, Metadata, Record};

pub struct STDOUTLogger;

impl log::Log for STDOUTLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, msg: &Record) {
        if self.enabled(msg.metadata()) {
            let s_level: String = match msg.level() {
                log::Level::Info => format!("{}", msg.level().as_str().bright_green()),
                log::Level::Warn => format!("{}", msg.level().as_str().yellow()),
                log::Level::Error => format!("{}", msg.level().as_str().bright_red()),
                log::Level::Debug => format!("{}", msg.level().as_str().bright_cyan()),
                log::Level::Trace => format!("{}", msg.level().as_str().cyan()),
            };

            println!("[{}] - {}: {}", Local::now().format("%d/%m/%Y %H:%M:%S"), s_level, msg.args());
        }
    }

    fn flush(&self) {}
}
