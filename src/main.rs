#![no_std]
#![no_main]

use embedded_hal::digital::OutputPin;
use rp2040_hal as hal;

mod panic;
mod usb;

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

    unsafe { hal::pac::NVIC::unmask(hal::pac::interrupt::USBCTRL_IRQ) };
    loop {
        heartbeat.set_high().ok();

        // if let Some(logger) = usb::LOGGER.get() {
        //     critical_section::with(|cs| {
        //         let mut logger = logger.borrow_ref_mut(cs);

        //         if logger.serial.dtr() {
        //             logger.serial.write(b"Hello world!\n\r").unwrap();
        //             logger.serial.flush().unwrap();
        //         }
        //     })
        // }
        cortex_m::asm::delay(12_000_000); // ~96ms pause between messages
        heartbeat.set_low().ok();
    }
}
