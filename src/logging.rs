use core::cell::RefCell;
use core::fmt::Write as FmtWrite;

use critical_section::{CriticalSection, Mutex};
use heapless::{Deque, String, Vec};
use log::{Level, LevelFilter, Log, Record};
use once_cell::sync::OnceCell;
use rp2040_hal::usb::UsbBus;
use static_cell::StaticCell;
use usbd_serial::{SerialPort, embedded_io::Write as IoWrite};

// 264 KB RAM * 45% hard limit = 119 KB
// 300 bytes = 119 KB = 396 records max
const LOG_BUFFER_CAPACITY: usize = 396;

static GLOBAL_LOGGER_STORAGE: StaticCell<Mutex<RefCell<SerialLogger>>> = StaticCell::new();
static GLOBAL_LOGGER: OnceCell<&'static mut Mutex<RefCell<SerialLogger>>> = OnceCell::new();
static LOGGING_SERVICE: UsbSerialLogger = UsbSerialLogger;

// 300 bytes
struct SerialLogRecord {
    // time_ms: u64,
    level: Level,
    target: String<32>,
    message: String<256>,
}

struct SerialLogger {
    level: LevelFilter,
    logs: Deque<SerialLogRecord, LOG_BUFFER_CAPACITY>,
}

struct UsbSerialLogger;

impl Log for UsbSerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if let Some(logger_mutex) = GLOBAL_LOGGER.get() {
            critical_section::with(|cs| {
                logger_mutex
                    .borrow(cs)
                    .borrow()
                    .level
                    .to_level()
                    .map(|l| l <= metadata.level())
                    .unwrap_or(false)
            })
        } else {
            false
        }
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
            target,
            message,
        };

        if let Some(logger_mutex) = GLOBAL_LOGGER.get() {
            critical_section::with(|cs| {
                logger_mutex
                    .borrow(cs)
                    .borrow_mut()
                    .logs
                    .push_back(new_record)
                    .ok();
            })
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(level: LevelFilter) -> Option<()> {
    let global_logger = GLOBAL_LOGGER_STORAGE.init(Mutex::new(RefCell::new(SerialLogger {
        level,
        logs: Deque::new(),
    })));

    GLOBAL_LOGGER.set(global_logger).ok()?;

    unsafe {
        if let Ok(()) = log::set_logger_racy(&LOGGING_SERVICE) {
            log::set_max_level_racy(level);
        }
    }

    Some(())
}

pub fn flush_logs<'a, 'b>(serial: &mut SerialPort<'a, UsbBus>, cs: CriticalSection<'b>) {
    let mut output_queue: Vec<SerialLogRecord, LOG_BUFFER_CAPACITY> = Vec::new();

    if let Some(logger_mutex) = GLOBAL_LOGGER.get() {
        while let Some(record) = logger_mutex.borrow_ref_mut(cs).logs.pop_front() {
            // Safe to ignore, since this cannot fail
            // We structured our output queue to have
            // the same capacity as the record queue,
            // therefore there is no chance for the
            output_queue.push(record).ok();
        }
    }

    for record in output_queue {
        let formatted_log =
            format_args!("[{}] <{}> {}", record.target, record.level, record.message);

        serial.write_fmt(formatted_log).ok();
    }
}
