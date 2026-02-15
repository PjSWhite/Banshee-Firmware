use rp2040_hal as hal;
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;

pub struct UsbCommandLine<'a> {
    pub cmdline: SerialPort<'a, hal::usb::UsbBus>,
    pub logger: SerialPort<'a, hal::usb::UsbBus>,
    device: UsbDevice<'a, hal::usb::UsbBus>,
    // allocator: UsbBusAllocator<hal::usb::UsbBus>,
}

impl<'a> UsbCommandLine<'a> {
    pub fn new(allocator: &'a UsbBusAllocator<hal::usb::UsbBus>) -> Self {
        let cmdline = SerialPort::new_with_interface_names(
            allocator,
            Some("OpenPanahon Commandline"),
            Some("cmd"),
        );
        let logger =
            SerialPort::new_with_interface_names(allocator, Some("OpenPanahon Log"), Some("log"));

        // NOTE: https://pid.codes/howto/
        //       We should make our own PID
        let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x2e8a, 0x000a))
            .strings(&[StringDescriptors::default()
                .manufacturer("CIT-U SHS SY 2526 STEM-12 ALTRUISM RESEARCH 4 GROUP 3")
                .product("OpenPanahon Station Node")
                .serial_number("PROTOTYPE-REV1")])
            .unwrap()
            .device_class(2)
            // .composite_with_iads()
            .build();

        Self {
            cmdline,
            logger,
            device,
        }
    }

    pub fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.cmdline, &mut self.logger])
    }
}
