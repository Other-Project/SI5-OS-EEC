use chrono::DateTime;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOG_BUFFER: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

struct TuiLogger;

impl log::Log for TuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now: DateTime<chrono::Local> = chrono::Local::now();
            let msg = format!("[{}] [{}] {}", now.format("%Y-%m-%d %H:%M:%S"), record.level(), record.args());
            let mut buf = LOG_BUFFER.lock().unwrap();
            buf.push(msg);
            
            // Limit buffer size
            if buf.len() > 100 {
                buf.remove(0);
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: TuiLogger = TuiLogger;

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
