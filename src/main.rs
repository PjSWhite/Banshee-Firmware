#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Debug};

use bme280::i2c::BME280;
use core::fmt::Write as FmtWrite;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use hal::fugit::RateExtU32;
use pms7003_rs::Pms7003Controller;
use rp2040_hal::{
    self as hal, fugit::MicrosDurationU32, prelude::*, timer::Alarm, uart::UartConfig,
};
use sgp40::Sgp40;

use rp2040_panic_usb_boot as _;

// use defmt as _;

// mod logging;
// mod panic;
mod serial;
mod usb;

#[used]
#[unsafe(link_section = ".boot2")]
static BOOTLOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// type I2cPin<T> = hal::gpio::Pin<T, hal::gpio::FunctionI2C, hal::gpio::PullUp>;
// type I2cPinConfiguration = (
//     I2cPin<hal::gpio::bank0::Gpio4>,
//     I2cPin<hal::gpio::bank0::Gpio5>,
// );
// type I2cBus = hal::I2C<hal::pac::I2C0, I2cPinConfiguration>;

// static mut SHARED_DEVICE: Option<
//     MyDevice<shared_bus::I2cProxy<shared_bus::CortexMMutex<SomeI2cBus>>>,
// > = None;
const CLOCK_SPEED: u32 = 12_000_000;

fn render_pm_readings<'a>(
    pms_result: pms7003_rs::DataFrameResult<'a>,
    serial_buffer: &mut serial::SerialBuffer,
) -> Result<(), core::fmt::Error> {
    if let Ok(reading) = pms_result {
        write!(
            serial_buffer,
            "PM1.0: {} ug/m3; PM2.5: {} ug/m3; PM10: {} ug/m3",
            reading.pm1_0_atm, reading.pm2_5_atm, reading.pm10_atm
        )
    } else {
        write!(serial_buffer, " PMS: {:?}", pms_result.unwrap_err())
    }
}

fn render_rhtbp_measurements<E>(
    measurements: &bme280::Measurements<E>,
    serial_buffer: &mut serial::SerialBuffer,
) -> Result<(), core::fmt::Error> {
    write!(
        serial_buffer,
        "T: {} deg C; RH: {} %; P: {} Pa",
        measurements.temperature, measurements.humidity, measurements.pressure,
    )
}

fn render_frame(
    frame: &[u8],
    serial_buffer: &mut serial::SerialBuffer,
) -> Result<(), core::fmt::Error> {
    for (pos, byte) in frame.iter().enumerate() {
        write!(serial_buffer, "{:X}", byte)?;

        if pos != frame.len() {
            write!(serial_buffer, " ")?;
        }
    }

    Ok(())
}

fn render_debug<E: Debug>(
    val: E,
    serial_buffer: &mut serial::SerialBuffer,
) -> Result<(), core::fmt::Error> {
    write!(serial_buffer, "{:?}", val)
}

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
    let mut cold_start_alarm = timer.alarm_3().unwrap();

    cold_start_alarm
        .schedule(MicrosDurationU32::secs(60))
        .unwrap();

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // clocks.usb_clock

    let mut heartbeat = pins.gpio25.into_push_pull_output();

    let uart = hal::uart::UartPeripheral::new(
        pac.UART0,
        (pins.gpio0.into_function(), pins.gpio1.into_function()),
        &mut pac.RESETS,
    )
    .enable(
        UartConfig::new(
            9600.Hz(),
            rp2040_hal::uart::DataBits::Eight,
            None,
            rp2040_hal::uart::StopBits::One,
        ),
        clocks.peripheral_clock.freq(),
    )
    .unwrap();

    let pms_alarm = timer.alarm_0().unwrap();
    let mut pms7003 = Pms7003Controller::new(uart, pms_alarm);

    let modbus = hal::uart::UartPeripheral::new(
        pac.UART1,
        (pins.gpio8.into_function(), pins.gpio9.into_function()),
        &mut pac.RESETS,
    )
    .enable(
        UartConfig::new(
            4800.Hz(),
            rp2040_hal::uart::DataBits::Eight,
            None,
            rp2040_hal::uart::StopBits::One,
        ),
        clocks.peripheral_clock.freq(),
    )
    .unwrap();

    let i2c_device = RefCell::new(hal::I2C::i2c0(
        pac.I2C0,
        pins.gpio4.reconfigure(),
        pins.gpio5.reconfigure(),
        400.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    ));
    let mut i2c_bme280 = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_device);
    let mut i2c_sgp40 = embedded_hal_bus::i2c::RefCellDevice::new(&i2c_device);

    let mut sgp40 = Sgp40::new(&mut i2c_sgp40, 0x59, timer);
    let mut bme280 = BME280::new_primary(&mut i2c_bme280);

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
    // logging::init_logger(log::LevelFilter::Info);

    // sensor preparation
    bme280.init(&mut timer).unwrap();

    pms7003.sleep().unwrap();
    timer.delay_ms(500);
    pms7003.wake().unwrap();
    timer.delay_ms(500);
    pms7003.passive().unwrap();

    unsafe { hal::pac::NVIC::unmask(hal::pac::interrupt::USBCTRL_IRQ) };
    // log::info!("Ready");

    // Wait for sensors to become ready/valid
    let command: [u8; _] = [0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x84, 0x39];
    let mut response: [u8; _] = [0; 7];
    loop {
        heartbeat.set_high().ok();
        timer.delay_ms(500);

        if cold_start_alarm.finished() {
            break;
        }

        heartbeat.set_low().ok();
        timer.delay_ms(500);

        if let Some(logger_mutex) = usb::LOGGER.get() {
            critical_section::with(|cs| {
                let mut logger = logger_mutex.borrow_ref_mut(cs);

                logger.serial.write(b"System Warmup\r\n").ok();
            })
        }
    }

    let mut serial_buffer = serial::SerialBuffer::default();
    loop {
        heartbeat.set_high().ok();
        pms7003.flush_data();

        modbus.write_full_blocking(&command);
        match modbus.read_full_blocking(&mut response) {
            Ok(()) => render_frame(&response, &mut serial_buffer),
            Err(err) => render_debug(err, &mut serial_buffer),
        }
        .unwrap();

        let pms_result = pms7003.read_passive();
        let measurements = bme280.measure(&mut timer).unwrap();
        let voc_index = sgp40
            .measure_raw_with_rht(
                (measurements.humidity * 1000.0) as u16,
                (measurements.temperature * 1000.0) as i16,
            )
            .and_then(|_| {
                sgp40.measure_voc_index_with_rht(
                    (measurements.humidity * 1000.0) as u16,
                    (measurements.temperature * 1000.0) as i16,
                )
            });

        if let Some(logger_mutex) = usb::LOGGER.get() {
            critical_section::with(|cs| {
                let mut logger_svc = logger_mutex.borrow_ref_mut(cs);
                if logger_svc.ready() {
                    // logger_svc.serial.write_fmt(msg).unwrap();

                    render_rhtbp_measurements(&measurements, &mut serial_buffer).unwrap();
                    write!(&mut serial_buffer, "; ").unwrap();
                    render_pm_readings(pms_result, &mut serial_buffer).unwrap();
                    write!(&mut serial_buffer, "; ").unwrap();

                    match voc_index {
                        Ok(voc) => {
                            write!(&mut serial_buffer, "VOC: {}", voc)
                        }
                        Err(err) => {
                            write!(&mut serial_buffer, "VOC ERROR: {:?}", err)
                        }
                    }
                    .unwrap();

                    logger_svc.serial.write(serial_buffer.buffer()).unwrap();
                    logger_svc.serial.write(b"\r\n").unwrap();
                    match logger_svc.serial.flush() {
                        Ok(_) | Err(usb_device::UsbError::WouldBlock) => (),
                        Err(err) => {
                            panic!("Error: {err:?}");
                        }
                    };

                    // let frame_dump = format_args!("Frame Dump:\n\r{}",);
                    // logging::flush_logs(&mut logger_svc.serial, cs);
                }
            })
        }

        serial_buffer.clear();
        // defmt::info!("Hello world!");
        timer.delay_ms(1000);
        // cortex_m::asm::delay(12_000_000); // ~96ms pause between messages
        heartbeat.set_low().ok();
    }
}
