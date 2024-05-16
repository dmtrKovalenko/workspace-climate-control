#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::clockcontrol, delay::delay, peripherals::peripherals, prelude::*};

extern crate alloc;
use core::mem::maybeuninit;

#[global_allocator]
static allocator: esp_alloc::espheap = esp_alloc::espheap::empty();

fn init_heap() {
    const heap_size: usize = 32 * 1024;
    static mut heap: maybeuninit<[u8; heap_size]> = maybeuninit::uninit();

    unsafe {
        allocator.init(heap.as_mut_ptr() as *mut u8, heap_size);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = peripherals::take();
    let system = peripherals.system.split();

    let clocks = clockcontrol::max(system.clock_control).freeze();
    let delay = delay::new(&clocks);
    init_heap();

    esp_println::logger::init_logger_from_env();

    let timer = esp_hal::timer::timergroup::new(peripherals.timg1, &clocks, none).timer0;
    let _init = esp_wifi::initialize(
        esp_wifi::espwifiinitfor::wifi,
        timer,
        esp_hal::rng::rng::new(peripherals.rng),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    loop {
        log::info!("hello world!");
        delay.delay(500.millis());
    }
}
