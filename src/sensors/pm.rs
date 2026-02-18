use hal::fugit::ExtU32;
use rp2040_hal::{self as hal, timer::Alarm};

use hal::timer::Alarm1 as AlarmSlot;
use pms7003_rs::{Pms7003Controller, TimerAlarm, driver::ReadResult};

type Pms7003UartDevice<P> = hal::uart::UartPeripheral<hal::uart::Enabled, hal::pac::UART0, P>;

struct Pms7003Alarm(AlarmSlot);

impl TimerAlarm for Pms7003Alarm {
    type Countdown = hal::fugit::MicrosDurationU32;
    type Result = Result<(), hal::timer::ScheduleAlarmError>;

    fn is_ready(&self) -> bool {
        self.0.finished()
    }

    fn schedule(&mut self, countdown: Self::Countdown) -> Self::Result {
        self.0.schedule(countdown)
    }
}

pub struct ParticulateMatterSensor<P: hal::uart::ValidUartPinout<hal::pac::UART0>> {
    sensor: Pms7003Controller<Pms7003UartDevice<P>, Pms7003Alarm>,
}
impl<P: hal::uart::ValidUartPinout<hal::pac::UART0>> ParticulateMatterSensor<P> {
    pub fn new(uart: Pms7003UartDevice<P>, alarm: AlarmSlot) -> Self {
        Self {
            sensor: Pms7003Controller::new(uart, Pms7003Alarm(alarm)),
        }
    }

    pub fn init(&mut self) {
        self.sensor.active().unwrap();
        self.sensor.passive().unwrap();

        self.sensor.timer_mut().0.schedule(30.secs()).unwrap();
    }

    pub fn try_read(&mut self) -> ReadResult<'_> {
        self.sensor.read_passive()
    }
}
