use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref PURPLE_BUFFER: Mutex<Vec<(String, log::Level, String)>> = Default::default();
}

pub struct PurpleDebugLogger;

impl log::Log for PurpleDebugLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        let purple_level = match record.level() {
            log::Level::Error => crate::PurpleDebugLevel::PURPLE_DEBUG_ERROR,
            log::Level::Warn => crate::PurpleDebugLevel::PURPLE_DEBUG_WARNING,
            log::Level::Info => crate::PurpleDebugLevel::PURPLE_DEBUG_INFO,
            _ => crate::PurpleDebugLevel::PURPLE_DEBUG_MISC,
        };

        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };
        let line = format!("[{}] {}\n", target, record.args());
        crate::debug(purple_level, "", &line);
    }

    fn flush(&self) {
        let buffer = {
            match PURPLE_BUFFER.lock() {
                Ok(mut buffer) => buffer.split_off(0),
                Err(_) => return,
            }
        };
        for (target, level, message) in buffer {
            log::log!(target: &target, level, "{}", message);
        }
    }
}
