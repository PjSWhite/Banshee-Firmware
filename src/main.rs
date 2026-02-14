#![no_std]
#![no_main]

use hal::fugit::ExtU32;
use panic_halt as _;
use rp2040_hal::{self as hal};
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// mod cmd;
// mod logging;
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

    // let mut timers = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

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

    // let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
    //     pac.USBCTRL_REGS,
    //     pac.USBCTRL_DPRAM,
    //     clocks.usb_clock,
    //     true,
    //     &mut pac.RESETS,
    // ));

    // let mut serial = SerialPort::new(&usb_bus);

    // // NOTE: https://pid.codes/howto/
    // //       We should make our own PID
    // let mut dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
    //     .strings(&[StringDescriptors::default()
    //         .manufacturer("CIT-U SHS SY 2526 STEM-12 ALTRUISM RESEARCH 4 GROUP 3")
    //         .product("OpenPanahon Station Node")
    //         .serial_number("NULL")])
    //     .unwrap()
    //     .device_class(2)
    //     .build();

    watchdog.start(500.millis());
    loop {
        usb.poll();

        // if dev.poll(&mut [&mut serial]) {
        //     let mut buf = [0; 32];

        //     match serial.read(&mut buf) {
        //         Err(_) | Ok(0) => (),
        //         Ok(count) => {
        //             buf.iter_mut()
        //                 .take(count)
        //                 .for_each(u8::make_ascii_uppercase);

        //             let reply_buffer = &buf[..count];
        //             serial.write(reply_buffer).unwrap();
        //         }
        //     }
        // }

        status.in_loop();

        watchdog.feed();
    }
}
