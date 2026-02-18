#![no_std]
#![no_main]

use hal::fugit::{ExtU32, RateExtU32};
use log::{error, info};
use panic_halt as _;
use rp2040_hal::{self as hal, Clock};
use usb_device::bus::UsbBusAllocator;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// mod cmd;
mod logging;
mod sensors;
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
    logging::init_logger(log::LevelFilter::Info);
    log::info!("Hello, World!");

    let pms7003_uart = hal::uart::UartPeripheral::new(
        pac.UART0,
        (pins.gpio0.into_function(), pins.gpio1.into_function()),
        &mut pac.RESETS,
    )
    .enable(
        hal::uart::UartConfig::new(
            9600.Hz(),
            hal::uart::DataBits::Eight,
            None,
            hal::uart::StopBits::One,
        ),
        clocks.peripheral_clock.freq(),
    )
    .unwrap();
    let pms7003_timer = time::with_timer_mut(|t| t.alarm_1().unwrap()).unwrap();

    let mut controller = sensors::pm::ParticulateMatterSensor::new(pms7003_uart, pms7003_timer);
    controller.init();

    // let pms_cmd_passive = [0x42, 0x4d, 0xe1, 0x00, 0x00, 0x01, 0x70];
    // let pms_cmd_passive_read = [0x42, 0x4d, 0xe2, 0x00, 0x00, 0x01, 0x71];
    // let pms_cmd_wake = [0x42, 0x4d, 0xe4, 0x00, 0x01, 0x01, 0x74];
    // let pms_cmd_sleep = [0x42, 0x4d, 0xe4, 0x00, 0x00, 0x01, 0x73];
    // let mut pms_resp = [0; 32];

    // let mut pms7003 = hal::uart::UartPeripheral::new(
    //     pac.UART0,
    //     (pins.gpio0.into_function(), pins.gpio1.into_function()),
    //     &mut pac.RESETS,
    // )
    // .enable(
    // hal::uart::UartConfig::new(
    //     9600.Hz(),
    //     hal::uart::DataBits::Eight,
    //     None,
    //     hal::uart::StopBits::One,
    // ),
    // clocks.peripheral_clock.freq(),
    // )
    // .unwrap();

    // pms7003.write_full_blocking(&pms_cmd_wake);
    // pms7003.write_full_blocking(&pms_cmd_passive);
    // log::info!("Initialized PM sensor, waiting 30s to initialize");

    // let mut timer = time::with_timer_mut(|t| t.alarm_3().unwrap()).unwrap();
    // let _ = timer.schedule(32.secs());

    watchdog.start(100.millis());
    loop {
        usb.poll();
        status.in_loop();

        match controller.try_read() {
            Ok(data) => {
                info!(
                    "PM1.0: {}, PM2.5: {}, PM10: {}",
                    data.pm1_0_atm, data.pm2_5_atm, data.pm10_atm
                );
            }
            Err(err) => match err {
                pms7003_rs::Error::Write(err) => {
                    error!("Controller Write Error: {}", err);
                }
                pms7003_rs::Error::Conversion => {
                    error!("Failed to interpret data frame")
                }
                pms7003_rs::Error::WarmUp => (),
            },
        }

        // if timer.finished() {
        //     timer.clear_interrupt();
        //     pms7003.write_full_blocking(&pms_cmd_passive_read);
        //     let _ = pms7003.read_full_blocking(&mut pms_resp);
        //     pms7003.write_full_blocking(&pms_cmd_sleep);

        //     log::info!("Got dump: {:x?}", pms_resp);
        // }

        logging::flush_logs(&mut usb.logger);
        watchdog.feed();
    }
}
