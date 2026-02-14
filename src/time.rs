use core::cell::RefCell;
use critical_section::Mutex;
use rp2040_hal as hal;

static GLOBAL_TIMER: Mutex<RefCell<Option<hal::Timer>>> = Mutex::new(RefCell::new(None));

pub fn init_timer(
    timers: hal::pac::TIMER,
    resets: &mut hal::pac::RESETS,
    clocks: &hal::clocks::ClocksManager,
) {
    let timers = hal::Timer::new(timers, resets, clocks);
    critical_section::with(|cs| {
        GLOBAL_TIMER.replace(cs, Some(timers));
    })
}

pub fn with_timer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&hal::Timer) -> R,
{
    critical_section::with(|cs| GLOBAL_TIMER.borrow_ref(cs).as_ref().map(f))
}

pub fn with_timer_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut hal::Timer) -> R,
{
    critical_section::with(|cs| GLOBAL_TIMER.borrow_ref_mut(cs).as_mut().map(f))
}
