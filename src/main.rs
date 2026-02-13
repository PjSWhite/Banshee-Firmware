#![no_std]
#![no_main]

use hal::fugit::ExtU32;
use panic_halt as _;
use rp2040_hal::{self as hal};

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

mod status;

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
    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut timers = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let led_pin = pins.gpio25.into_push_pull_output();
    let mut status = status::StatusPin::new(led_pin, timers.alarm_0().unwrap());

    watchdog.start(500.millis());
    loop {
        status.in_loop();

        watchdog.feed();
    }
}
