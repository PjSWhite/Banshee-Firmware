// use core::pin::Pin;
use core::cell::RefCell;
use once_cell::sync::OnceCell;

use critical_section::Mutex;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;
// use rp2040_hal::pac::USBCTRL_REGS;
use rp2040_hal as hal;

use hal::pac::interrupt;
use hal::usb::UsbBus;

// pub static USB_ALLOCATOR: OnceCell<AllocatorWrapper> = OnceCell::new();
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

/// # THIS IS EXTREMELY RADIOACTIVE
/// Any interactions with this struct, and
/// consequently `USB_ALLOCATOR`, **requires
/// an environment where interrupts are disabled**
///
/// Please do take care in
// pub struct AllocatorWrapper(pub *const UsbBusAllocator<UsbBus>);

// unsafe impl Sync for AllocatorWrapper {}
// unsafe impl Send for AllocatorWrapper {}

pub struct UsbLogger<'a> {
    pub serial: SerialPort<'a, UsbBus>,
    device: UsbDevice<'a, UsbBus>,
    // allocator: UsbBusAllocator<UsbBus>,
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
    // USB_ALLOCATOR
    //     .set(AllocatorWrapper(alloc as *const _))
    //     .ok()?;

    Some(())
}
