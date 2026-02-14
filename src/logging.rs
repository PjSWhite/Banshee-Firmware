use core::cell::RefCell;
use core::{fmt::Write, str::FromStr};

use rp2040_hal as hal;

use critical_section::Mutex;
use heapless::{Deque, String};
use log::{Level, Log, Record};
use usbd_serial::SerialPort;

type SerialLoggerPort<'a> = SerialPort<'a, hal::usb::UsbBus>;

const LOG_BUFFER_CAPACITY: usize = 64; // theoretical max: 550 (~25% of RAM)
static GLOBAL_LOGGER: Mutex<RefCell<Option<SerialLogger<'static>>>> =
    Mutex::new(RefCell::new(None));

#[derive(Clone)]
struct SerialLogRecord {
    time_ms: u64,
    level: Level,
    target: String<32>,
    message: String<64>,
}

struct SerialLogger<'a> {
    level: Level,
    serial_port: SerialLoggerPort<'a>,
    logs: Deque<SerialLogRecord, LOG_BUFFER_CAPACITY>,
}

struct SerialLoggingService;

impl Log for SerialLoggingService {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        critical_section::with(|cs| {
            if let Some(logger) = GLOBAL_LOGGER.borrow(cs).borrow().as_ref() {
                logger.level <= metadata.level()
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
        write!(&mut message, "{}", record.args());

        let target_raw = record.target();
        let mut target = String::new();

        if target_raw.len() <= target.capacity() {
            target.push_str(target_raw).ok();
        } else {
            let available = target.capacity().saturating_sub(4);
            writeln!(&mut target, "{}...", &target_raw[..available]);
        }

        let new_record = SerialLogRecord {
            level: record.level(),
            time_ms: 0,
            target,
            message,
        };

        critical_section::with(|cs| {
            if let Some(logger) = GLOBAL_LOGGER.borrow(cs).borrow_mut().as_mut() {
                logger.logs.push_back(new_record);
            }
        });
    }

    fn flush(&self) {
        todo!()
    }
}

pub fn new<'a>(level: Level, serial_port: SerialLoggerPort<'static>) {
    let logger = SerialLogger {
        level,
        serial_port,
        logs: Deque::new(),
    };

    critical_section::with(|cs| GLOBAL_LOGGER.replace(cs, Some(logger)));
}
