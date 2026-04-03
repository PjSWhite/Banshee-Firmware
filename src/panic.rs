use core::panic::PanicInfo;
use rp2040_hal::pac::Peripherals;

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
fn panic(_: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    let pac = unsafe { Peripherals::steal() };
    // let sio = hal::Sio::new(pac.SIO);

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
        beep(SHORT);
        beep(SHORT);
        beep(SHORT);
        cortex_m::asm::delay(GAP);
        // O: ---
        beep(LONG);
        beep(LONG);
        beep(LONG);
        cortex_m::asm::delay(GAP);
        // S: ...
        beep(SHORT);
        beep(SHORT);
        beep(SHORT);
        cortex_m::asm::delay(WORD_GAP);
    }
}
