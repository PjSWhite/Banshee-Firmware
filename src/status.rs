use embedded_hal::digital::StatefulOutputPin;
use hal::fugit::ExtU32;
use rp2040_hal::{self as hal, timer::Alarm};

type ActPin =
    hal::gpio::Pin<hal::gpio::bank0::Gpio25, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;
type ActAlarm = hal::timer::Alarm0;

pub struct StatusPin {
    act_led: ActPin,
    alarm: ActAlarm,
}

impl StatusPin {
    pub fn new(act_led: ActPin, alarm: ActAlarm) -> Self {
        Self { act_led, alarm }
    }

    pub fn in_loop(&mut self) {
        if self.alarm.finished() {
            let _ = self.alarm.schedule(250.millis());
            self.act_led.toggle();
        }
    }
}
