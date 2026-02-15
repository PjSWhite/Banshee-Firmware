use core::cell::RefCell;
use core::fmt::Write as FmtWrite;

use rp2040_hal as hal;

use critical_section::Mutex;
use heapless::{Deque, String};
use log::{Level, LevelFilter, Log, Record};
use usbd_serial::{SerialPort, embedded_io::Write as EioWrite};

use crate::time::with_timer;

type SerialLoggerPort<'a> = SerialPort<'a, hal::usb::UsbBus>;

const LOG_BUFFER_CAPACITY: usize = 64; // theoretical max: 550 (~25% of RAM)
static GLOBAL_LOGGER: Mutex<RefCell<Option<SerialLogger>>> = Mutex::new(RefCell::new(None));
static LOGGING_SERVICE: SerialLoggingService = SerialLoggingService;

#[derive(Clone)]
struct SerialLogRecord {
    time_ms: u64,
    level: Level,
    target: String<32>,
    message: String<64>,
}

struct SerialLogger {
    level: LevelFilter,
    logs: Deque<SerialLogRecord, LOG_BUFFER_CAPACITY>,
}

struct SerialLoggingService;

impl Log for SerialLoggingService {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        critical_section::with(|cs| {
            if let Some(logger) = GLOBAL_LOGGER.borrow(cs).borrow().as_ref() {
                logger
                    .level
                    .to_level()
                    .and_then(|l| Some(l <= metadata.level()))
                    .or(Some(false))
                    .unwrap()
            } else {
                false
            }
        })
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut message = String::new();
        let _ = write!(&mut message, "{}", record.args());

        let target_raw = record.target();
        let mut target = String::new();

        if target_raw.len() <= target.capacity() {
            let _ = target.push_str(target_raw);
        } else {
            let available = target.capacity().saturating_sub(4);
            let _ = writeln!(&mut target, "{}...", &target_raw[..available]);
        }

        let new_record = SerialLogRecord {
            level: record.level(),
            time_ms: with_timer(|timer| timer.get_counter().ticks() / 1000).unwrap_or(0),
            target,
            message,
        };

        critical_section::with(|cs| {
            if let Some(logger) = GLOBAL_LOGGER.borrow(cs).borrow_mut().as_mut() {
                let _ = logger.logs.push_back(new_record);
            }
        });
    }

    fn flush(&self) {}
}

pub fn init_logger(level: LevelFilter) {
    let logger = SerialLogger {
        logs: Deque::new(),
        level,
    };

    critical_section::with(|cs| GLOBAL_LOGGER.replace(cs, Some(logger)));

    unsafe {
        if let Ok(()) = log::set_logger_racy(&LOGGING_SERVICE) {
            log::set_max_level_racy(level);
        }
    }
}

pub fn flush_logs<'a>(serial_port: &mut SerialLoggerPort<'a>) {
    critical_section::with(|cs| {
        // NOTE: Output buffer has 128 characters max
        // let mut message: String<128> = String::new();
        let mut global_logger = GLOBAL_LOGGER.borrow_ref_mut(cs);
        let logger = global_logger.as_mut().unwrap();

        if !serial_port.dtr() {
            // log::info!("Connection with Logging Interface established");
            // log::info!("Dumping everything :3");
            return;
        }

        while let Some(record) = logger.logs.pop_front() {
            let args = format_args!(
                "[{}] [{}/{}] {}\r\n",
                record.time_ms,
                record.level.as_str(),
                record.target,
                record.message
            );

            let _ = serial_port.write_fmt(args);
        }

        let _ = serial_port.flush();
    });
}
