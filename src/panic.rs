use core::panic::PanicInfo;
use hal::gpio::{FunctionSio, Pin, PullDown, SioOutput};
use rp2040_hal::{self as hal, pac::Peripherals};

// GPIO25 is the onboard LED on Pico -- type alias for clarity
// type LedPin = Pin<hal::gpio::bank0::Gpio25, FunctionSio<SioOutput>, PullDown>;

// We need a way to get the buzzer pin in the panic handler.
// It must be a Mutex<RefCell<Option<...>>> so we can safely set it
// from main before panics can occur.
use core::cell::RefCell;
use critical_section::Mutex;

// Adjust the pin number to wherever your buzzer is wired
type BuzzerPin = Pin<hal::gpio::bank0::Gpio15, FunctionSio<SioOutput>, PullDown>;

pub static PANIC_BUZZER: Mutex<RefCell<Option<BuzzerPin>>> = Mutex::new(RefCell::new(None));
const BUZZER_BIT: u32 = 1 << 15;
const LED_BIT: u32 = 1 << 25;

const CLOCK_SPEED: u32 = 125_000;
const SHORT: u32 = CLOCK_SPEED * 5;
const LONG: u32 = CLOCK_SPEED * 10;
const WORD_GAP: u32 = CLOCK_SPEED * 3;

// I want to explicitly document that this takes
// roughly one second
#[allow(clippy::identity_op)]
const GAP: u32 = CLOCK_SPEED * 1;

pub fn set_panic_buzzer(pin: BuzzerPin) {
    critical_section::with(|cs| {
        PANIC_BUZZER.borrow_ref_mut(cs).replace(pin);
    });
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    let pac = unsafe { Peripherals::steal() };
    // let sio = hal::Sio::new(pac.SIO);

    let gpio_out_set = &pac.SIO.gpio_out_set();
    let gpio_out_clr = &pac.SIO.gpio_out_clr();

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
