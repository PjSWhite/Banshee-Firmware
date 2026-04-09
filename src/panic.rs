use core::panic::PanicInfo;
use rp2040_hal as hal;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;
use usbd_serial::embedded_io::Write as EioWrite;

use super::CLOCK_SPEED;

// Adjust the pin number to wherever your buzzer is wired
const BUZZER_BIT: u32 = 1 << 15;
const LED_BIT: u32 = 1 << 25;

const SHORT: u32 = CLOCK_SPEED * 3;
const LONG: u32 = CLOCK_SPEED * 5;
const WORD_GAP: u32 = CLOCK_SPEED * 10;

// I want to explicitly document that this takes
// roughly one second
#[allow(clippy::identity_op)]
const GAP: u32 = CLOCK_SPEED * 1;

#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    // let mut panic_buffer: [u8; 256] = [0; 256];

    // format_panic(panic, &mut panic_buffer);

    let pac = unsafe { hal::pac::Peripherals::steal() };

    // let usb_clock = unsafe { core::mem::zeroed::<hal::clocks::UsbClock>() };
    // let mut device = None;
    // let mut serial = None;
    // if let Some(alloc_ptr) = crate::usb::USB_ALLOCATOR.get() {
    //     let alloc: &'static UsbBusAllocator<hal::usb::UsbBus> = unsafe { &*alloc_ptr.0 };

    //     pac.RESETS.reset().modify(|_, w| w.usbctrl().set_bit());
    //     pac.RESETS.reset().modify(|_, w| w.usbctrl().clear_bit());
    //     // Wait for reset to complete
    //     while pac.RESETS.reset_done().read().usbctrl().bit_is_clear() {}

    //     serial = Some(SerialPort::new(alloc));
    //     device = Some(
    //         UsbDeviceBuilder::new(alloc, UsbVidPid(0x2e8a, 0x000a))
    //             .strings(&[StringDescriptors::default()
    //                 .manufacturer("CIT-U SHS SY 2526 STEM-12 ALTRUISM RESEARCH 4 GROUP 3")
    //                 .product("OpenPanahon Station Node")
    //                 .serial_number("0xDBE9")])
    //             .unwrap()
    //             .device_class(2)
    //             .build(),
    //     );
    // }

    let gpio_oe_set = &pac.SIO.gpio_oe_set();
    let gpio_out_set = &pac.SIO.gpio_out_set();
    let gpio_out_clr = &pac.SIO.gpio_out_clr();

    unsafe {
        gpio_oe_set.write(|w| w.bits(BUZZER_BIT | LED_BIT));
    };

    pac.IO_BANK0
        .gpio(15)
        .gpio_ctrl()
        .write(|w| w.funcsel().sio());
    pac.IO_BANK0
        .gpio(25)
        .gpio_ctrl()
        .write(|w| w.funcsel().sio());

    let beep = |duration: u32| {
        gpio_out_set.write(|w| unsafe { w.bits(BUZZER_BIT | LED_BIT) });
        cortex_m::asm::delay(duration);
        gpio_out_clr.write(|w| unsafe { w.bits(BUZZER_BIT | LED_BIT) });
        cortex_m::asm::delay(GAP);
    };

    loop {
        // poll_if_available(serial.as_mut(), device.as_mut());

        beep(SHORT);
        beep(SHORT);
        beep(SHORT);
        // write_panic_message(serial.as_mut(), &panic_buffer);

        cortex_m::asm::delay(GAP);
        // O: ---
        beep(LONG);
        beep(LONG);
        beep(LONG);
        // write_panic_message(serial.as_mut(), &panic_buffer);

        cortex_m::asm::delay(GAP);
        // S: ...
        beep(SHORT);
        beep(SHORT);
        beep(SHORT);
        // write_panic_message(serial.as_mut(), &panic_buffer);

        cortex_m::asm::delay(WORD_GAP);
    }
}

// fn poll_if_available(
//     serial: Option<&mut SerialPort<hal::usb::UsbBus>>,
//     device: Option<&mut UsbDevice<hal::usb::UsbBus>>,
// ) -> bool {
//     if let (Some(device), Some(serial)) = (device, serial) {
//         device.poll(&mut [serial])
//     } else {
//         false
//     }
// }

// fn format_panic(panic: &PanicInfo, mut buf: &mut [u8]) {
//     let message = panic.message().as_str().unwrap_or("<unknown message>");

//     match panic.location() {
//         Some(loc) => write!(
//             buf,
//             "Firmware panic at {:?} line {}, col {}: {}\r\n",
//             loc.file(),
//             loc.line(),
//             loc.column(),
//             message
//         ),
//         None => write!(buf, "Firmware panic: {}\r\n", message),
//     }
//     .ok();
// }

// fn write_panic_message(serial: Option<&mut SerialPort<hal::usb::UsbBus>>, panic_message: &[u8]) {
//     if serial.is_none() {
//         return;
//     }

//     let serial = serial.unwrap();
//     if !serial.dtr() {
//         return;
//     }

//     serial.write(panic_message).ok();
//     serial.flush().ok();
// }
