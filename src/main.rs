#![no_std]
#![no_main]

use hal::fugit::ExtU32;
use panic_halt as _;
use rp2040_hal::{self as hal};
use usb_device::bus::UsbBusAllocator;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// mod cmd;
mod logging;
mod status;
mod time;
mod usb;

#[hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    time::init_timer(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let led_pin = pins.gpio25.into_push_pull_output();
    let mut status = status::StatusPin::new(led_pin);

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut usb = usb::UsbCommandLine::new(&usb_bus);
    logging::init_logger(log::LevelFilter::Info);

    log::info!("Hello, World!");

    watchdog.start(500.millis());
    loop {
        usb.poll();
        status.in_loop();

        logging::flush_logs(&mut usb.logger);
        watchdog.feed();
    }
}
