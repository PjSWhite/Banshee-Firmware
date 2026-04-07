#![no_std]
#![no_main]

use core::cell::RefCell;

use bme280::i2c::BME280;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use hal::fugit::RateExtU32;
use rp2040_hal as hal;
use sgp40::Sgp40;
use usbd_serial::embedded_io::Write;

use defmt as _;

mod logging;
mod panic;
mod usb;

#[used]
#[unsafe(link_section = ".boot2")]
static BOOTLOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

type I2cPin<T> = hal::gpio::Pin<T, hal::gpio::FunctionI2C, hal::gpio::PullUp>;
type I2cPinConfiguration = (
    I2cPin<hal::gpio::bank0::Gpio4>,
    I2cPin<hal::gpio::bank0::Gpio5>,
);
type I2cBus = hal::I2C<hal::pac::I2C0, I2cPinConfiguration>;

// static mut SHARED_DEVICE: Option<
//     MyDevice<shared_bus::I2cProxy<shared_bus::CortexMMutex<SomeI2cBus>>>,
// > = None;
const CLOCK_SPEED: u32 = 12_000_000;

#[hal::entry]
unsafe fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        CLOCK_SPEED,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut heartbeat = pins.gpio25.into_push_pull_output();

    let i2c_device = RefCell::new(hal::I2C::i2c0(
        pac.I2C0,
        pins.gpio4.reconfigure(),
        pins.gpio5.reconfigure(),
        400.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    ));
    let mut i2c_bme280 = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_device);
    let mut i2c_sgp40 = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_device);

    let mut sgp40 = Sgp40::new(&mut i2c_sgp40, 0x59, timer);
    let mut bme280 = BME280::new_primary(&mut i2c_bme280);

    let usb_bus = hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );

    // We want this to be loud if the USB device couldnt
    // be initialized properly.
    // Mechanically, the reason can be one of two:
    //  1) Allocator is already initialized (cortex_m::singleton! returned None)
    //  2) usb::LOGGER already set (OnceCell::set() returned None)
    // All of these possible ways initialization could
    // fail are all cause by calling usb::init_usb twice
    // in the program flow
    //
    // TODO: Probably do another initialization
    // routine in the panic handler so we can
    // extract the panic message?
    usb::init_usb(usb_bus).unwrap();
    // logging::init_logger(log::LevelFilter::Info);

    // sensor preparation
    bme280.init(&mut timer).unwrap();

    unsafe { hal::pac::NVIC::unmask(hal::pac::interrupt::USBCTRL_IRQ) };
    // log::info!("Ready");

    cortex_m::asm::delay(24_000_000);
    loop {
        heartbeat.set_high().ok();

        let measurements = bme280.measure(&mut timer).unwrap();
        let voc_index = sgp40
            .measure_voc_index_with_rht(
                (measurements.humidity * 1000.0) as u16,
                (measurements.temperature * 1000.0) as i16,
            )
            .unwrap();

        if let Some(logger_mutex) = usb::LOGGER.get() {
            critical_section::with(|cs| {
                let mut logger_svc = logger_mutex.borrow_ref_mut(cs);

                if logger_svc.ready() {
                    let msg = format_args!(
                        "T: {} deg C; RH: {} %; P: {} Pa; VOC: {}\n\r",
                        measurements.temperature,
                        measurements.humidity,
                        measurements.pressure,
                        voc_index
                    );
                    logger_svc.serial.write_fmt(msg);

                    // logging::flush_logs(&mut logger_svc.serial, cs);
                }
            })
        }

        defmt::info!("Hello world!");
        timer.delay_ms(1000);
        // cortex_m::asm::delay(12_000_000); // ~96ms pause between messages
        heartbeat.set_low().ok();
    }
}
