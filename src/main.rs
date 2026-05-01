#![no_std]
#![no_main]
use core::fmt::Write as FmtWrite;

use embedded_hal::delay::DelayNs;
use rp2040_hal as hal;
use rp2040_hal::Clock;
use rp2040_hal::fugit::RateExtU32 as _;
use rp2040_panic_usb_boot as _;

use sx127x_lora::LoRa;
use sx127x_lora::MODE;

use crate::serial::SerialBuffer;

mod sensor;
mod serial;
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
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let lora_miso = pins.gpio16.into_function::<hal::gpio::FunctionSpi>();
    let lora_mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
    let lora_sclk = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let lora_csel = pins
        .gpio17
        .into_push_pull_output_in_state(rp2040_hal::gpio::PinState::High);
    let lora_rset = pins
        .gpio20
        .into_push_pull_output_in_state(rp2040_hal::gpio::PinState::High);

    let lora_spi = hal::Spi::<_, _, _, 8>::new(pac.SPI0, (lora_mosi, lora_miso, lora_sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1_000_000u32.Hz(),
        MODE,
    );
    let mut lora = LoRa::new(lora_spi, lora_csel, lora_rset, 433, timer).unwrap();

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
    usb::init_usb(usb_bus).unwrap();

    lora.set_tx_power(17, 1).unwrap();

    unsafe { hal::pac::NVIC::unmask(hal::pac::interrupt::USBCTRL_IRQ) };

    // let mut serial_buffer = ser
    let mut lora_packet = SerialBuffer::<255>::default();
    let mut packet_number: u32 = 0;
    loop {
        write!(
            &mut lora_packet,
            "Hello world! This packet number {}",
            packet_number
        )
        .unwrap();

        packet_number = packet_number.wrapping_add(1);
        lora.transmit_payload_busy(lora_packet.inner(), lora_packet.len())
            .unwrap();
        timer.delay_ms(1000);
        lora_packet.clear();
    }
}
