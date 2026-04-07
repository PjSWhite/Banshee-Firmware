use portable_atomic::{AtomicBool, Ordering};

static TAKEN: AtomicBool = AtomicBool::new(false);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[defmt::global_logger]
struct UsbDefmtLogger;

#[allow(static_mut_refs)]
unsafe impl defmt::Logger for UsbDefmtLogger {
    fn acquire() {
        // disable interrupts, claim the encoder
        cortex_m::interrupt::disable();
        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger already acquired");
        }
        TAKEN.store(true, Ordering::Relaxed);
        unsafe { ENCODER.start_frame(write_encoded) };
    }

    unsafe fn flush() {}

    unsafe fn release() {
        unsafe { ENCODER.end_frame(write_encoded) };
        TAKEN.store(false, Ordering::Relaxed);
        unsafe { cortex_m::interrupt::enable() };
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe { ENCODER.write(bytes, write_encoded) };
    }
}

fn write_encoded(bytes: &[u8]) {
    if let Some(logger_mutex) = crate::usb::LOGGER.get() {
        critical_section::with(|cs| {
            let mut logger = logger_mutex.borrow_ref_mut(cs);

            logger.serial.write(bytes).ok();
            // if logger.ready() {
            // }
        })
    }
}

// #[defmt::timestamp]
// fn timestamp() -> u64 {
//     // Access the RP2040 timer peripheral
//     // For a quick test, you can return 0, but logs won't have timing info
//     0
// }
