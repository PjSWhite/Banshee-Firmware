use core::cell::RefCell;
use once_cell::sync::OnceCell;

use critical_section::Mutex;
use rp2040_hal as hal;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;

use hal::pac::interrupt;
use hal::usb::UsbBus;

pub static LOGGER: OnceCell<Mutex<RefCell<UsbLogger<'static>>>> = OnceCell::new();

#[interrupt]
fn USBCTRL_IRQ() {
    if let Some(logger) = LOGGER.get() {
        critical_section::with(|cs| {
            let mut logger = logger.borrow_ref_mut(cs);

            logger.poll();
        })
    }
}

pub struct UsbLogger<'a> {
    pub serial: SerialPort<'a, UsbBus>,
    device: UsbDevice<'a, UsbBus>,
}

impl<'a> UsbLogger<'a> {
    fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.serial])
    }

    pub fn ready(&self) -> bool {
        self.serial.dtr()
    }
}

pub fn init_usb(bus: UsbBus) -> Option<()> {
    let alloc = cortex_m::singleton!(
        USB_ALLOCATOR: UsbBusAllocator<UsbBus> = UsbBusAllocator::new(bus)
    )?;

    let serial = SerialPort::new(alloc);
    let device = UsbDeviceBuilder::new(alloc, UsbVidPid(0x2e8a, 0x000a))
        .strings(&[StringDescriptors::default()
            .manufacturer("CIT-U SHS SY 2526 STEM-12 ALTRUISM RESEARCH 4 GROUP 3")
            .product("OpenPanahon Station Node")
            .serial_number("0xDBE9")])
        .unwrap()
        .device_class(2)
        .build();

    LOGGER
        .set(Mutex::new(RefCell::new(UsbLogger { serial, device })))
        .ok()?;

    Some(())
}
