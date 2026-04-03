#![no_std]
#![no_main]

use embedded_hal::digital::OutputPin;
use rp2040_hal as hal;

mod panic;

#[used]
#[unsafe(link_section = ".boot2")]
static BOOTLOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

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

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut heartbeat = pins.gpio25.into_push_pull_output();

    // panic!("this is test");

    loop {
        heartbeat.set_high().ok();
        // cortex_m::asm::nop();
        heartbeat.set_low().ok();
    }
}
